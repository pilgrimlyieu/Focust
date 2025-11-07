//! Shared state management for all schedulers and monitors
//!
//! This module provides a centralized state management system that coordinates
//! pause/resume behavior across all schedulers (Break and Attention) and monitors.
//!
//! # Design Goals
//!
//! - **Single Source of Truth**: All pause reasons are managed in one place
//! - **Consistency**: Break and Attention schedulers share the same pause state
//! - **Session Protection**: Monitors don't interfere during active Break/Attention sessions
//! - **Thread Safety**: All state access is synchronized via `RwLock`

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
            tracing::info!("Scheduler paused: {reason}");
        } else if inserted {
            tracing::info!("Additional pause reason: {reason}");
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
            tracing::info!("Scheduler resumed: all pause reasons cleared");
        } else if removed {
            tracing::info!("Pause reason removed: {reason} (still paused by other reasons)");
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
        if !self.in_break_session {
            self.in_break_session = true;
            self.break_session_start = Some(Instant::now());
            tracing::debug!("Break session started");
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
                tracing::debug!("Break session ended (duration: {duration:?})");
            }
            self.break_session_start = None;
        }
    }

    /// Start an attention session
    ///
    /// Called when an attention reminder begins.
    pub fn start_attention_session(&mut self) {
        if !self.in_attention_session {
            self.in_attention_session = true;
            self.attention_session_start = Some(Instant::now());
            tracing::debug!("Attention session started");
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
                tracing::debug!("Attention session ended (duration: {duration:?})");
            }
            self.attention_session_start = None;
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
        state.end_break_session();
        assert!(!state.in_break_session());
    }
}
