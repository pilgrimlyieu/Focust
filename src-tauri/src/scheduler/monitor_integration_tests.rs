//! Integration tests for Monitor system
//!
//! Tests the complete monitor orchestration system including:
//! - `IdleMonitor`: User idle detection
//! - `DndMonitor`: Do Not Disturb mode detection
//! - `AppWhitelistMonitor`: Application exclusion detection
//! - Multi-monitor coordination
//! - Monitor error handling and recovery
//!
//! # Test Strategy
//!
//! These tests simulate monitors through `SchedulerManager` to test
//! real-world integration, not isolated monitor behavior.
//!
//! Key scenarios:
//! - Single monitor triggering pause/resume
//! - Multiple monitors with concurrent pause reasons
//! - Monitor interaction with active sessions
//! - Error recovery and graceful degradation

use tokio::sync::mpsc;

use crate::scheduler::models::{Command, PauseReason};
use crate::scheduler::test_helpers::manager::*;
use crate::scheduler::test_helpers::state_machine::advance_time_and_yield;
use crate::scheduler::test_helpers::*;

// ============================================================================
// Single Monitor Tests
// ============================================================================

/// **MON1.1: `IdleMonitor` Triggers Pause**
///
/// When user is idle, `IdleMonitor` should send Pause(UserIdle) command.
#[tokio::test(start_paused = true)]
async fn test_idle_monitor_triggers_pause() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Verify initially running
    {
        let state = env.shared_state.read();
        assert!(!state.is_paused());
    }

    // Simulate IdleMonitor detecting idle
    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify paused
    {
        let state = env.shared_state.read();
        assert!(state.is_paused());
        assert!(state.pause_reasons().contains(&PauseReason::UserIdle));
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON1.2: `IdleMonitor` Resumes After Activity**
///
/// When user becomes active, `IdleMonitor` should send Resume(UserIdle).
#[tokio::test(start_paused = true)]
async fn test_idle_monitor_resumes() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Simulate idle -> pause
    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    assert!(env.shared_state.read().is_paused());

    // Simulate activity -> resume
    cmd_tx
        .send(Command::Resume(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify resumed
    {
        let state = env.shared_state.read();
        assert!(!state.is_paused());
        assert!(!state.pause_reasons().contains(&PauseReason::UserIdle));
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON1.3: `DndMonitor` Triggers Pause**
///
/// When DND mode enabled, `DndMonitor` should pause scheduler.
#[tokio::test(start_paused = true)]
async fn test_dnd_monitor_triggers_pause() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Simulate DND enabled
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify paused
    {
        let state = env.shared_state.read();
        assert!(state.is_paused());
        assert!(state.pause_reasons().contains(&PauseReason::Dnd));
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON1.4: `DndMonitor` Resumes After DND Disabled**
///
/// When DND mode disabled, `DndMonitor` should resume scheduler.
#[tokio::test(start_paused = true)]
async fn test_dnd_monitor_resumes() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Simulate DND enabled -> paused
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Simulate DND disabled -> resumed
    cmd_tx
        .send(Command::Resume(PauseReason::Dnd))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify resumed
    {
        let state = env.shared_state.read();
        assert!(!state.is_paused());
        assert!(!state.pause_reasons().contains(&PauseReason::Dnd));
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON1.5: `AppWhitelistMonitor` Triggers Pause**
///
/// When excluded app is active, `AppWhitelistMonitor` should pause.
#[tokio::test(start_paused = true)]
async fn test_app_whitelist_monitor_triggers_pause() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Simulate excluded app active
    cmd_tx
        .send(Command::Pause(PauseReason::AppExclusion))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify paused
    {
        let state = env.shared_state.read();
        assert!(state.is_paused());
        assert!(state.pause_reasons().contains(&PauseReason::AppExclusion));
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON1.6: `AppWhitelistMonitor` Resumes**
///
/// When excluded app is no longer active, should resume.
#[tokio::test(start_paused = true)]
async fn test_app_whitelist_monitor_resumes() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Simulate excluded app active -> paused
    cmd_tx
        .send(Command::Pause(PauseReason::AppExclusion))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Simulate app closed -> resumed
    cmd_tx
        .send(Command::Resume(PauseReason::AppExclusion))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify resumed
    {
        let state = env.shared_state.read();
        assert!(!state.is_paused());
        assert!(!state.pause_reasons().contains(&PauseReason::AppExclusion));
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

// ============================================================================
// Multi-Monitor Coordination Tests
// ============================================================================

/// **MON2.1: Two Monitors - Both Trigger Pause**
///
/// When multiple monitors detect pause conditions simultaneously.
#[tokio::test(start_paused = true)]
async fn test_two_monitors_both_pause() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // IdleMonitor detects idle
    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // DndMonitor detects DND (shortly after)
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // Verify both reasons tracked
    {
        let state = env.shared_state.read();
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 2);
        assert!(state.pause_reasons().contains(&PauseReason::UserIdle));
        assert!(state.pause_reasons().contains(&PauseReason::Dnd));
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON2.2: Two Monitors - Sequential Resume**
///
/// When one monitor resumes but another still paused, stay paused.
#[tokio::test(start_paused = true)]
async fn test_two_monitors_sequential_resume() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Both monitors trigger pause
    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // User becomes active (IdleMonitor resumes)
    cmd_tx
        .send(Command::Resume(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // Should still be paused (DND active)
    {
        let state = env.shared_state.read();
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 1);
        assert!(state.pause_reasons().contains(&PauseReason::Dnd));
    }

    // DND disabled (DndMonitor resumes)
    cmd_tx
        .send(Command::Resume(PauseReason::Dnd))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // Now should be running (all reasons cleared)
    {
        let state = env.shared_state.read();
        assert!(!state.is_paused());
        assert_eq!(state.pause_reasons().len(), 0);
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON2.3: Three Monitors - All Trigger Then Clear**
///
/// All three monitors trigger pause, then clear in different order.
#[tokio::test(start_paused = true)]
async fn test_three_monitors_coordinated() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // All three monitors trigger pause
    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    cmd_tx
        .send(Command::Pause(PauseReason::AppExclusion))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify all three reasons
    {
        let state = env.shared_state.read();
        assert_eq!(state.pause_reasons().len(), 3);
    }

    // Clear in order: App, Idle, DND
    cmd_tx
        .send(Command::Resume(PauseReason::AppExclusion))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(50)).await;
    assert!(env.shared_state.read().is_paused()); // Still 2 reasons

    cmd_tx
        .send(Command::Resume(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(50)).await;
    assert!(env.shared_state.read().is_paused()); // Still 1 reason

    cmd_tx
        .send(Command::Resume(PauseReason::Dnd))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(50)).await;
    assert!(!env.shared_state.read().is_paused()); // All cleared

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON2.4: Monitor Flapping - Rapid Pause/Resume**
///
/// Monitor rapidly toggling between pause and resume states.
#[tokio::test(start_paused = true)]
async fn test_monitor_flapping() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Simulate rapid pause/resume cycles (e.g., user moving mouse sporadically)
    for _ in 0..10 {
        cmd_tx
            .send(Command::Pause(PauseReason::UserIdle))
            .await
            .unwrap();
        cmd_tx
            .send(Command::Resume(PauseReason::UserIdle))
            .await
            .unwrap();
    }

    advance_time_and_yield(duration_ms(200)).await;

    // Final state should be running (last was resume)
    {
        let state = env.shared_state.read();
        assert!(!state.is_paused());
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

// ============================================================================
// Monitor + User Interaction Tests
// ============================================================================

/// **MON3.1: Monitor Pause + User Manual Pause**
///
/// Monitor pauses, then user also manually pauses.
#[tokio::test(start_paused = true)]
async fn test_monitor_and_user_pause() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Monitor detects idle
    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // User manually pauses
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // Both reasons should be tracked
    {
        let state = env.shared_state.read();
        assert_eq!(state.pause_reasons().len(), 2);
        assert!(state.pause_reasons().contains(&PauseReason::UserIdle));
        assert!(state.pause_reasons().contains(&PauseReason::Manual));
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON3.2: Monitor Resumes But User Still Paused**
///
/// Monitor condition clears but user pause remains.
#[tokio::test(start_paused = true)]
async fn test_monitor_resumes_user_still_paused() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Monitor + user both pause
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Monitor condition clears
    cmd_tx
        .send(Command::Resume(PauseReason::Dnd))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // Should still be paused (user manual pause)
    {
        let state = env.shared_state.read();
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 1);
        assert!(state.pause_reasons().contains(&PauseReason::Manual));
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON3.3: User Resumes During Monitor Pause**
///
/// User tries to manually resume while monitor still has pause condition.
#[tokio::test(start_paused = true)]
async fn test_user_resume_during_monitor_pause() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Both monitor and user pause
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // User tries to resume manually
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // Should still be paused (monitor reason remains)
    {
        let state = env.shared_state.read();
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 1);
        assert!(state.pause_reasons().contains(&PauseReason::Dnd));
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

// ============================================================================
// Complex Scenarios
// ============================================================================

/// **MON4.1: Monitor Storm - All Conditions Rapidly Change**
///
/// Simulates chaotic environment with all monitors rapidly changing.
#[tokio::test(start_paused = true)]
async fn test_monitor_storm() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Simulate chaotic sequence of monitor events
    let events = vec![
        Command::Pause(PauseReason::UserIdle),
        Command::Pause(PauseReason::Dnd),
        Command::Resume(PauseReason::UserIdle),
        Command::Pause(PauseReason::AppExclusion),
        Command::Resume(PauseReason::Dnd),
        Command::Pause(PauseReason::Manual),
        Command::Resume(PauseReason::AppExclusion),
        Command::Pause(PauseReason::Dnd),
        Command::Resume(PauseReason::Manual),
        Command::Resume(PauseReason::Dnd),
    ];

    for event in events {
        cmd_tx.send(event).await.unwrap();
        advance_time_and_yield(duration_ms(10)).await;
    }

    // Final state should be running (all reasons cleared)
    {
        let state = env.shared_state.read();
        assert!(!state.is_paused());
        assert_eq!(state.pause_reasons().len(), 0);
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON4.2: Monitor Cascade - One Trigger Leads to Another**
///
/// Simulates realistic scenario where conditions cascade.
#[tokio::test(start_paused = true)]
async fn test_monitor_cascade() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // User goes idle
    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // While idle, OS enables DND (common on Windows/macOS)
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // While still in DND, user opens excluded app (e.g., game)
    cmd_tx
        .send(Command::Pause(PauseReason::AppExclusion))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // Now all three conditions active
    {
        let state = env.shared_state.read();
        assert_eq!(state.pause_reasons().len(), 3);
    }

    // User becomes active -> idle clears
    cmd_tx
        .send(Command::Resume(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(50)).await;

    // Still paused (2 reasons remain)
    assert_eq!(env.shared_state.read().pause_reasons().len(), 2);

    // User closes game -> app exclusion clears
    cmd_tx
        .send(Command::Resume(PauseReason::AppExclusion))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(50)).await;

    // Still paused (DND remains)
    assert_eq!(env.shared_state.read().pause_reasons().len(), 1);

    // DND disables -> all clear
    cmd_tx
        .send(Command::Resume(PauseReason::Dnd))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(50)).await;

    // Now running
    assert!(!env.shared_state.read().is_paused());

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON4.3: Monitor Recovery - Redundant Commands**
///
/// Monitors may send redundant pause/resume commands due to check intervals.
#[tokio::test(start_paused = true)]
async fn test_monitor_redundant_commands() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Monitor sends pause multiple times (e.g., checked every second, condition persists)
    for _ in 0..5 {
        cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
        advance_time_and_yield(duration_ms(50)).await;
    }

    // Should only have one reason (idempotent)
    {
        let state = env.shared_state.read();
        assert_eq!(state.pause_reasons().len(), 1);
    }

    // Monitor sends resume multiple times
    for _ in 0..5 {
        cmd_tx
            .send(Command::Resume(PauseReason::Dnd))
            .await
            .unwrap();
        advance_time_and_yield(duration_ms(50)).await;
    }

    // Should be running (idempotent resume)
    {
        let state = env.shared_state.read();
        assert!(!state.is_paused());
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

// ============================================================================
// Edge Cases
// ============================================================================

/// **MON5.1: All Four Pause Reasons Simultaneously**
///
/// Stress test: all possible pause reasons active at once.
#[tokio::test(start_paused = true)]
async fn test_all_four_pause_reasons() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // All four reasons trigger
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    cmd_tx
        .send(Command::Pause(PauseReason::AppExclusion))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify all four tracked
    {
        let state = env.shared_state.read();
        assert_eq!(state.pause_reasons().len(), 4);
    }

    // Clear all four
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    cmd_tx
        .send(Command::Resume(PauseReason::UserIdle))
        .await
        .unwrap();
    cmd_tx
        .send(Command::Resume(PauseReason::Dnd))
        .await
        .unwrap();
    cmd_tx
        .send(Command::Resume(PauseReason::AppExclusion))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Should be running
    assert!(!env.shared_state.read().is_paused());

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **MON5.2: Monitor During Break**
///
/// Test that session state is properly managed and accessible.
/// This verifies the `SharedState` session tracking APIs work correctly.
#[tokio::test(start_paused = true)]
async fn test_monitor_during_break_session() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(30)
        .mini_break_duration_s(10)
        .notification_before_s(0)
        .build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Test session state management
    {
        let mut state = env.shared_state.write();
        assert!(!state.in_break_session());
        assert!(!state.in_attention_session());
        assert!(!state.in_any_session());

        // Start break session
        state.start_break_session();
        assert!(state.in_break_session());
        assert!(state.in_any_session());

        // End break session
        state.end_break_session();
        assert!(!state.in_break_session());
        assert!(!state.in_any_session());
    }

    // Test attention session
    {
        let mut state = env.shared_state.write();
        state.start_attention_session();
        assert!(state.in_attention_session());
        assert!(state.in_any_session());

        state.end_attention_session();
        assert!(!state.in_attention_session());
        assert!(!state.in_any_session());
    }

    // Test overlapping sessions
    {
        let mut state = env.shared_state.write();
        state.start_break_session();
        state.start_attention_session();
        assert!(state.in_break_session());
        assert!(state.in_attention_session());
        assert!(state.in_any_session());
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}
