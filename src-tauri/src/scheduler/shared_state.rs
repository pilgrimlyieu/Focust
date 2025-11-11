//! Shared state management for all schedulers and monitors
//!
//! # Overview
//!
//! This module provides a centralized state management system that coordinates
//! pause/resume behavior and session tracking across all schedulers and monitors.
//!
//! # Architecture
//!
//! ```text
//!                     ┌─────────────────┐
//!                     │  SharedState    │
//!                     │                 │
//!                     │ pause_reasons   │ ◄─── Manager (add/remove)
//!                     │ in_break_...    │ ◄─── BreakScheduler (start/end)
//!                     │ in_attention_.. │ ◄─── AttentionTimer (start/end)
//!                     └────────┬────────┘
//!                              │
//!                 ┌────────────┼────────────┐
//!                 ▼            ▼            ▼
//!          IdleMonitor   DndMonitor   AppWhitelist
//!          (read only)   (read only)  (read only)
//! ```
//!
//! # Design Goals
//!
//! 1. **Single Source of Truth**: All pause reasons managed in one place
//! 2. **Consistency**: All schedulers share the same pause state
//! 3. **Session Protection**: Monitors can query session state to avoid interference
//! 4. **Thread Safety**: All state access synchronized via `RwLock`
//! 5. **Composability**: Multiple pause reasons can coexist
//!
//! # Core Concepts
//!
//! ## Pause Reasons
//!
//! Multiple pause reasons can be active simultaneously (e.g., user idle + DND mode).
//! The scheduler is paused when **any** reason exists, and resumes only when **all**
//! reasons are cleared.
//!
//! ```text
//! pause_reasons = {}                    → Scheduler: Running
//! pause_reasons = {UserIdle}            → Scheduler: Paused
//! pause_reasons = {UserIdle, Dnd}       → Scheduler: Paused
//! pause_reasons = {Dnd}                 → Scheduler: Paused
//! pause_reasons = {}                    → Scheduler: Running
//! ```
//!
//! ## Sessions
//!
//! Sessions represent active user interactions (break or attention window).
//! During a session, monitors should avoid triggering pause commands to prevent
//! self-interference (e.g., break window triggering DND mode).
//!
//! **Session Types:**
//! - **Break Session**: Short/long break window is open
//! - **Attention Session**: Attention reminder is displayed
//!
//! **Why Track Sessions?**
//! - Break windows may trigger system DND mode
//! - We don't want DND monitor to pause the scheduler during breaks
//! - Monitors check `in_any_session()` to filter out such events
//!
//! # Usage Patterns
//!
//! ## For Monitors (Read-Only)
//!
//! ```rust,ignore
//! // Check if in session (avoid self-triggering)
//! if shared_state.read().in_any_session() {
//!     return Ok(MonitorAction::None);
//! }
//!
//! // Check if already paused (avoid duplicate commands)
//! if shared_state.read().is_paused() {
//!     // Don't send another Pause command
//! }
//! ```
//!
//! ## For Manager (Write)
//!
//! ```rust,ignore
//! // Handle Pause command
//! let should_pause = shared_state.write().add_pause_reason(reason);
//! if should_pause {
//!     // First pause reason, forward to schedulers
//! }
//!
//! // Handle Resume command
//! let should_resume = shared_state.write().remove_pause_reason(reason);
//! if should_resume {
//!     // All reasons cleared, forward to schedulers
//! }
//! ```
//!
//! ## For Schedulers (Write)
//!
//! ```rust,ignore
//! // Mark break session start
//! shared_state.write().start_break_session();
//! create_break_windows(...);
//! // ... break window open ...
//! shared_state.write().end_break_session();
//!
//! // Mark attention session start
//! shared_state.write().start_attention_session();
//! show_attention_prompt(...);
//! // ... prompt displayed ...
//! shared_state.write().end_attention_session();
//! ```
//!
//! # Thread Safety
//!
//! - Uses `Arc<RwLock<SharedSchedulerState>>` for shared ownership
//! - Multiple readers can access simultaneously (`.read()`)
//! - Writers get exclusive access (`.write()`)
//! - Lock guards are short-lived (don't hold across `.await` points)
//!
//! # Performance
//!
//! - `RwLock` allows concurrent reads (all monitors can check simultaneously)
//! - Writes are rare (only on pause/resume transitions and session changes)
//! - `HashSet` operations are O(1) average case
//!
//! # See Also
//!
//! - `scheduler::manager`] - Command routing and state management
//! - `monitors::dnd` - Example of session-aware monitoring

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;

use super::models::PauseReason;

/// Shared state between all schedulers and monitors
///
/// This state is wrapped in `Arc<RwLock<>>` for thread-safe shared access.
/// Multiple readers can access simultaneously, but writers get exclusive access.
#[derive(Debug)]
pub struct SharedSchedulerState {
    /// Active pause reasons
    ///
    /// When this set is non-empty, the scheduler is paused.
    /// Multiple reasons can be active simultaneously.
    pause_reasons: HashSet<PauseReason>,

    /// Whether currently in a break session (mini or long)
    ///
    /// Set to true when a break starts, false when it finishes.
    /// During a break session, DND changes are ignored to prevent
    /// the break window from pausing itself.
    in_break_session: bool,

    /// Whether currently in an attention reminder session
    ///
    /// Set to true when attention starts, false when it finishes.
    in_attention_session: bool,

    /// When the break session started (for logging/debugging)
    break_session_start: Option<Instant>,

    /// When the attention session started (for logging/debugging)
    attention_session_start: Option<Instant>,
}

impl SharedSchedulerState {
    /// Create a new shared scheduler state
    #[must_use]
    pub fn new() -> Self {
        Self {
            pause_reasons: HashSet::new(),
            in_break_session: false,
            in_attention_session: false,
            break_session_start: None,
            attention_session_start: None,
        }
    }

    /// Add a pause reason
    ///
    /// Returns `true` if this caused a state transition from Running to Paused
    /// (i.e., this was the first pause reason added).
    ///
    /// # Example State Transitions
    ///
    /// ```text
    /// Reasons: {}        -> add(Idle)  -> Reasons: {Idle}       [Returns: true]
    /// Reasons: {Idle}    -> add(Dnd)   -> Reasons: {Idle, Dnd}  [Returns: false]
    /// ```
    pub fn add_pause_reason(&mut self, reason: PauseReason) -> bool {
        let was_running = self.pause_reasons.is_empty();
        let inserted = self.pause_reasons.insert(reason);

        if was_running {
            tracing::info!("🔴 SharedState: Scheduler PAUSED (first reason: {reason})");
            tracing::trace!("SharedState: pause_reasons = {:?}", self.pause_reasons);
        } else if inserted {
            tracing::info!(
                "SharedState: Added pause reason {reason} (already paused, total: {})",
                self.pause_reasons.len()
            );
            tracing::trace!("SharedState: pause_reasons = {:?}", self.pause_reasons);
        } else {
            tracing::trace!("SharedState: Duplicate pause reason {reason} (ignored)");
        }

        was_running
    }

    /// Remove a pause reason
    ///
    /// Returns `true` if this caused a state transition from Paused to Running
    /// (i.e., this was the last pause reason removed).
    ///
    /// # Example State Transitions
    ///
    /// ```text
    /// Reasons: {Idle, Dnd} -> remove(Idle) -> Reasons: {Dnd}  [Returns: false]
    /// Reasons: {Dnd}       -> remove(Dnd)  -> Reasons: {}     [Returns: true]
    /// ```
    pub fn remove_pause_reason(&mut self, reason: PauseReason) -> bool {
        let removed = self.pause_reasons.remove(&reason);
        let is_now_running = self.pause_reasons.is_empty();

        if is_now_running {
            tracing::info!("🟢 SharedState: Scheduler RESUMED (all pause reasons cleared)");
            tracing::trace!("SharedState: pause_reasons = {:?}", self.pause_reasons);
        } else if removed {
            tracing::info!(
                "SharedState: Removed pause reason {reason} (still paused by {} other reason(s))",
                self.pause_reasons.len()
            );
            tracing::trace!(
                "SharedState: remaining pause_reasons = {:?}",
                self.pause_reasons
            );
        } else {
            tracing::trace!("SharedState: Tried to remove non-existent reason {reason}");
        }

        is_now_running
    }

    /// Check if scheduler is currently paused
    ///
    /// Returns `true` if there are any active pause reasons.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        !self.pause_reasons.is_empty()
    }

    /// Get the set of active pause reasons (for debugging/status display)
    #[must_use]
    pub fn pause_reasons(&self) -> &HashSet<PauseReason> {
        &self.pause_reasons
    }

    /// Check if in any session (break or attention)
    ///
    /// This is used by monitors (especially `DndMonitor`) to avoid interfering
    /// with active break/attention sessions.
    #[must_use]
    pub fn in_any_session(&self) -> bool {
        self.in_break_session || self.in_attention_session
    }

    /// Check if in a break session
    #[must_use]
    pub fn in_break_session(&self) -> bool {
        self.in_break_session
    }

    /// Check if in an attention session
    #[must_use]
    pub fn in_attention_session(&self) -> bool {
        self.in_attention_session
    }

    /// Start a break session
    ///
    /// Called when a break (mini or long) begins.
    pub fn start_break_session(&mut self) {
        if self.in_break_session {
            tracing::trace!("SharedState: Break session already active (ignored)");
        } else {
            self.in_break_session = true;
            self.break_session_start = Some(Instant::now());
            tracing::debug!("🪟 SharedState: Break session STARTED");
            tracing::trace!(
                "SharedState: in_break_session = true, in_attention_session = {}",
                self.in_attention_session
            );
        }
    }

    /// End a break session
    ///
    /// Called when a break (mini or long) finishes.
    pub fn end_break_session(&mut self) {
        if self.in_break_session {
            self.in_break_session = false;
            if let Some(start) = self.break_session_start {
                let duration = start.elapsed();
                tracing::debug!("🪟 SharedState: Break session ENDED (duration: {duration:?})");
            } else {
                tracing::debug!("🪟 SharedState: Break session ENDED");
            }
            tracing::trace!(
                "SharedState: in_break_session = false, in_attention_session = {}",
                self.in_attention_session
            );
            self.break_session_start = None;
        } else {
            tracing::trace!("SharedState: No active break session to end (ignored)");
        }
    }

    /// Start an attention session
    ///
    /// Called when an attention reminder begins.
    pub fn start_attention_session(&mut self) {
        if self.in_attention_session {
            tracing::trace!("SharedState: Attention session already active (ignored)");
        } else {
            self.in_attention_session = true;
            self.attention_session_start = Some(Instant::now());
            tracing::debug!("💡 SharedState: Attention session STARTED");
            tracing::trace!(
                "SharedState: in_attention_session = true, in_break_session = {}",
                self.in_break_session
            );
        }
    }

    /// End an attention session
    ///
    /// Called when an attention reminder finishes.
    pub fn end_attention_session(&mut self) {
        if self.in_attention_session {
            self.in_attention_session = false;
            if let Some(start) = self.attention_session_start {
                let duration = start.elapsed();
                tracing::debug!("💡 SharedState: Attention session ENDED (duration: {duration:?})");
            } else {
                tracing::debug!("💡 SharedState: Attention session ENDED");
            }
            tracing::trace!(
                "SharedState: in_attention_session = false, in_break_session = {}",
                self.in_break_session
            );
            self.attention_session_start = None;
        } else {
            tracing::trace!("SharedState: No active attention session to end (ignored)");
        }
    }
}

impl Default for SharedSchedulerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for the shared state wrapper
pub type SharedState = Arc<RwLock<SharedSchedulerState>>;

/// Create a new shared scheduler state
#[must_use]
pub fn create_shared_state() -> SharedState {
    Arc::new(RwLock::new(SharedSchedulerState::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pause_reason_management() {
        let mut state = SharedSchedulerState::new();

        // Initially not paused
        assert!(!state.is_paused());
        assert!(state.pause_reasons().is_empty());

        // Add first reason -> transition to Paused
        assert!(state.add_pause_reason(PauseReason::UserIdle));
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 1);

        // Add second reason -> still Paused, no transition
        assert!(!state.add_pause_reason(PauseReason::Dnd));
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 2);

        // Remove first reason -> still Paused
        assert!(!state.remove_pause_reason(PauseReason::UserIdle));
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 1);

        // Remove last reason -> transition to Running
        assert!(state.remove_pause_reason(PauseReason::Dnd));
        assert!(!state.is_paused());
        assert!(state.pause_reasons().is_empty());
    }

    #[test]
    fn test_duplicate_pause_reason() {
        let mut state = SharedSchedulerState::new();

        // Add same reason twice
        assert!(state.add_pause_reason(PauseReason::Manual));
        assert!(!state.add_pause_reason(PauseReason::Manual)); // No transition
        assert_eq!(state.pause_reasons().len(), 1);
    }

    #[test]
    fn test_remove_nonexistent_reason() {
        let mut state = SharedSchedulerState::new();
        state.add_pause_reason(PauseReason::UserIdle);

        // Remove a reason that wasn't added
        assert!(!state.remove_pause_reason(PauseReason::Dnd));
        assert!(state.is_paused()); // Still paused by UserIdle
    }

    #[test]
    fn test_session_management() {
        let mut state = SharedSchedulerState::new();

        // Initially not in any session
        assert!(!state.in_any_session());
        assert!(!state.in_break_session());
        assert!(!state.in_attention_session());

        // Start break session
        state.start_break_session();
        assert!(state.in_any_session());
        assert!(state.in_break_session());
        assert!(!state.in_attention_session());

        // End break session
        state.end_break_session();
        assert!(!state.in_any_session());

        // Start attention session
        state.start_attention_session();
        assert!(state.in_any_session());
        assert!(!state.in_break_session());
        assert!(state.in_attention_session());

        // End attention session
        state.end_attention_session();
        assert!(!state.in_any_session());
    }

    #[test]
    fn test_both_sessions_active() {
        let mut state = SharedSchedulerState::new();

        state.start_break_session();
        state.start_attention_session();

        assert!(state.in_any_session());
        assert!(state.in_break_session());
        assert!(state.in_attention_session());

        state.end_break_session();
        assert!(state.in_any_session()); // Still in attention session

        state.end_attention_session();
        assert!(!state.in_any_session());
    }

    #[test]
    fn test_idempotent_session_operations() {
        let mut state = SharedSchedulerState::new();

        // Starting same session multiple times
        state.start_break_session();
        state.start_break_session();
        assert!(state.in_break_session());

        // Ending same session multiple times
        state.end_break_session();
        assert!(!state.in_break_session());
        state.end_break_session();
        assert!(!state.in_break_session());
    }

    #[test]
    fn test_multiple_pause_reasons_simultaneously() {
        let mut state = SharedSchedulerState::new();

        // Add three different pause reasons
        assert!(state.add_pause_reason(PauseReason::UserIdle)); // Transition to Paused
        assert!(!state.add_pause_reason(PauseReason::Dnd)); // No transition
        assert!(!state.add_pause_reason(PauseReason::Manual)); // No transition

        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 3);

        // Remove middle reason - still paused
        assert!(!state.remove_pause_reason(PauseReason::Dnd));
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 2);

        // Remove all remaining reasons one by one
        assert!(!state.remove_pause_reason(PauseReason::Manual));
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 1);

        // Remove last reason - transition to Running
        assert!(state.remove_pause_reason(PauseReason::UserIdle));
        assert!(!state.is_paused());
        assert!(state.pause_reasons().is_empty());
    }

    #[test]
    fn test_pause_during_active_sessions() {
        let mut state = SharedSchedulerState::new();

        // Start both sessions
        state.start_break_session();
        state.start_attention_session();
        assert!(state.in_any_session());

        // Pause should not affect session state
        state.add_pause_reason(PauseReason::Dnd);
        assert!(state.is_paused());
        assert!(state.in_break_session());
        assert!(state.in_attention_session());

        // Remove pause reason - sessions still active
        state.remove_pause_reason(PauseReason::Dnd);
        assert!(!state.is_paused());
        assert!(state.in_break_session());
        assert!(state.in_attention_session());
    }

    #[test]
    fn test_session_operations_during_pause() {
        let mut state = SharedSchedulerState::new();

        // Pause first
        state.add_pause_reason(PauseReason::Manual);
        assert!(state.is_paused());

        // Session operations work during pause
        state.start_break_session();
        assert!(state.in_break_session());
        assert!(state.is_paused()); // Still paused

        state.end_break_session();
        assert!(!state.in_break_session());
        assert!(state.is_paused()); // Still paused

        // Resume
        state.remove_pause_reason(PauseReason::Manual);
        assert!(!state.is_paused());
    }

    #[test]
    fn test_rapid_pause_resume_cycles() {
        let mut state = SharedSchedulerState::new();

        // Rapid cycling should maintain consistency
        for _ in 0..10 {
            assert!(state.add_pause_reason(PauseReason::UserIdle));
            assert!(state.is_paused());
            assert!(state.remove_pause_reason(PauseReason::UserIdle));
            assert!(!state.is_paused());
        }

        // Final state should be consistent
        assert!(!state.is_paused());
        assert!(state.pause_reasons().is_empty());
    }

    #[test]
    fn test_alternating_pause_reasons() {
        let mut state = SharedSchedulerState::new();

        // Add reason A
        state.add_pause_reason(PauseReason::UserIdle);
        assert!(state.is_paused());

        // Add reason B while A is active
        state.add_pause_reason(PauseReason::Dnd);
        assert_eq!(state.pause_reasons().len(), 2);

        // Remove reason A - still paused by B
        state.remove_pause_reason(PauseReason::UserIdle);
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 1);

        // Add reason A back while B is active
        state.add_pause_reason(PauseReason::UserIdle);
        assert_eq!(state.pause_reasons().len(), 2);

        // Remove reason B - still paused by A
        state.remove_pause_reason(PauseReason::Dnd);
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 1);
    }

    #[test]
    fn test_session_overlap_scenarios() {
        let mut state = SharedSchedulerState::new();

        // Scenario 1: Start break, then attention, end break
        state.start_break_session();
        state.start_attention_session();
        state.end_break_session();
        assert!(state.in_attention_session());
        assert!(!state.in_break_session());

        // Scenario 2: Start another break while attention is active
        state.start_break_session();
        assert!(state.in_break_session());
        assert!(state.in_attention_session());

        // Scenario 3: End attention, break remains
        state.end_attention_session();
        assert!(state.in_break_session());
        assert!(!state.in_attention_session());

        // Clean up
        state.end_break_session();
        assert!(!state.in_any_session());
    }

    #[test]
    fn test_session_end_without_start() {
        let mut state = SharedSchedulerState::new();

        // Ending a session that was never started should be idempotent
        state.end_break_session();
        state.end_attention_session();
        assert!(!state.in_any_session());
    }

    #[test]
    fn test_complex_interleaved_operations() {
        let mut state = SharedSchedulerState::new();

        // Complex sequence: pause, session, unpause, session, pause
        state.add_pause_reason(PauseReason::Dnd);
        state.start_break_session();
        state.add_pause_reason(PauseReason::UserIdle);
        state.start_attention_session();
        state.remove_pause_reason(PauseReason::Dnd);
        state.end_break_session();
        state.remove_pause_reason(PauseReason::UserIdle);
        state.end_attention_session();

        // Final state should be clean
        assert!(!state.is_paused());
        assert!(!state.in_any_session());
        assert!(state.pause_reasons().is_empty());
    }

    #[test]
    fn test_all_pause_reasons_types() {
        let mut state = SharedSchedulerState::new();

        // Test all enum variants
        let all_reasons = vec![
            PauseReason::Manual,
            PauseReason::UserIdle,
            PauseReason::Dnd,
            PauseReason::AppExclusion,
        ];

        // Add all reasons
        for reason in &all_reasons {
            state.add_pause_reason(*reason);
            assert!(state.is_paused());
        }
        assert_eq!(state.pause_reasons().len(), all_reasons.len());

        // Remove all reasons
        for reason in &all_reasons {
            assert!(state.is_paused());
            state.remove_pause_reason(*reason);
        }
        assert!(state.pause_reasons().is_empty());
        assert!(!state.is_paused());
    }
}
