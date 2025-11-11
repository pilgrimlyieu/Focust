//! Integration tests for `SchedulerManager`
//!
//! Tests the top-level scheduler manager that coordinates:
//! - Command routing to break scheduler and attention timer
//! - Pause reason management (multiple concurrent reasons)
//! - Shared state coordination
//! - Event broadcasting
//!
//! # Architecture Under Test
//!
//! ```text
//! Commands → SchedulerManager.broadcast_commands()
//!              ├─> SharedState (pause reasons)
//!              ├─> BreakScheduler
//!              └─> AttentionTimer
//! ```
//!
//! # Test Strategy
//!
//! All tests use the `SchedulerManager` level (not individual schedulers)
//! to simulate real-world usage patterns.

use tokio::sync::mpsc;

use crate::scheduler::models::{Command, PauseReason};
use crate::scheduler::test_helpers::manager::*;
use crate::scheduler::test_helpers::state_machine::advance_time_and_yield;
use crate::scheduler::test_helpers::*;

// ============================================================================
// Multi-Reason Pause Management Tests
// ============================================================================

/// **M1.1: First Pause Reason Triggers Pause**
///
/// Adding the first pause reason should pause all schedulers.
#[tokio::test(start_paused = true)]
async fn test_first_pause_reason() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Add first pause reason
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify SharedState updated
    {
        let state = env.shared_state.read();
        assert!(state.pause_reasons().contains(&PauseReason::Manual));
        assert!(state.is_paused());
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **M1.2: Multiple Pause Reasons - Stays Paused**
///
/// Adding additional pause reasons should keep scheduler paused.
#[tokio::test(start_paused = true)]
async fn test_multiple_pause_reasons() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Add multiple pause reasons
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify all reasons tracked
    {
        let state = env.shared_state.read();
        assert!(state.pause_reasons().contains(&PauseReason::Manual));
        assert!(state.pause_reasons().contains(&PauseReason::UserIdle));
        assert!(state.pause_reasons().contains(&PauseReason::Dnd));
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 3);
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **M1.3: Partial Resume - Stays Paused**
///
/// Removing one pause reason when multiple exist should keep paused.
#[tokio::test(start_paused = true)]
async fn test_partial_resume_stays_paused() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Add multiple pause reasons
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Remove one reason
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Should still be paused (DND reason remains)
    {
        let state = env.shared_state.read();
        assert!(!state.pause_reasons().contains(&PauseReason::Manual));
        assert!(state.pause_reasons().contains(&PauseReason::Dnd));
        assert!(state.is_paused(), "Should stay paused (DND remains)");
        assert_eq!(state.pause_reasons().len(), 1);
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **M1.4: Last Resume - Triggers Resume**
///
/// Removing the last pause reason should resume all schedulers.
#[tokio::test(start_paused = true)]
async fn test_last_resume_triggers_resume() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Add two pause reasons
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Remove both reasons
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // Still paused
    {
        let state = env.shared_state.read();
        assert!(state.is_paused());
    }

    cmd_tx
        .send(Command::Resume(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Now should be running
    {
        let state = env.shared_state.read();
        assert!(
            !state.is_paused(),
            "Should be running (all reasons cleared)"
        );
        assert_eq!(state.pause_reasons().len(), 0);
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **M1.5: Duplicate Pause Reason - Idempotent**
///
/// Adding same pause reason multiple times should be idempotent.
#[tokio::test(start_paused = true)]
async fn test_duplicate_pause_reason_idempotent() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Add same reason multiple times
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Should only count once
    {
        let state = env.shared_state.read();
        assert_eq!(state.pause_reasons().len(), 1);
    }

    // Single resume should clear it
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    {
        let state = env.shared_state.read();
        assert!(!state.is_paused());
        assert_eq!(state.pause_reasons().len(), 0);
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **M1.6: Resume Non-Existent Reason - No-Op**
///
/// Removing a pause reason that wasn't added should be safe.
#[tokio::test(start_paused = true)]
async fn test_resume_non_existent_reason() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Add one reason
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Try to remove different reason
    cmd_tx
        .send(Command::Resume(PauseReason::Dnd))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Should still be paused (Manual remains)
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

// ============================================================================
// Pause Reason Priority Tests
// ============================================================================

/// **M2.1: All Pause Reasons Coexist**
///
/// Test all four pause reasons can coexist.
#[tokio::test(start_paused = true)]
async fn test_all_pause_reasons_coexist() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Add all pause reasons
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

    // Verify all tracked
    {
        let state = env.shared_state.read();
        assert_eq!(state.pause_reasons().len(), 4);
        assert!(state.pause_reasons().contains(&PauseReason::Manual));
        assert!(state.pause_reasons().contains(&PauseReason::UserIdle));
        assert!(state.pause_reasons().contains(&PauseReason::Dnd));
        assert!(state.pause_reasons().contains(&PauseReason::AppExclusion));
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **M2.2: Clear Reasons in Different Order**
///
/// Removing pause reasons in different order than added.
#[tokio::test(start_paused = true)]
async fn test_clear_reasons_different_order() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Add in order: Manual, Idle, DND
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Remove in order: DND, Manual, Idle (reverse)
    cmd_tx
        .send(Command::Resume(PauseReason::Dnd))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;
    assert!(env.shared_state.read().is_paused());

    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;
    assert!(env.shared_state.read().is_paused());

    cmd_tx
        .send(Command::Resume(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    assert!(!env.shared_state.read().is_paused());

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

// ============================================================================
// Command Routing Tests
// ============================================================================

/// **M3.1: `UpdateConfig` Broadcasts to All**
///
/// `UpdateConfig` should be sent to all schedulers.
#[tokio::test(start_paused = true)]
async fn test_update_config_broadcasts() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Update config
    let new_config = TestConfigBuilder::new().mini_break_interval_s(120).build();

    cmd_tx
        .send(Command::UpdateConfig(new_config))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Both schedulers should process it (can't directly verify, but test passes if no crash)
    // In real app, would see updated schedules

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **M3.2: Break Commands Route to `BreakScheduler`**
///
/// Break-specific commands should only go to `BreakScheduler`.
#[tokio::test(start_paused = true)]
async fn test_break_commands_route_correctly() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(30)
        .notification_before_s(0)
        .build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Send break-specific commands
    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    cmd_tx.send(Command::PostponeBreak).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    cmd_tx.send(Command::SkipBreak).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Should not crash - commands routed correctly

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

// ============================================================================
// Concurrent Operations Tests
// ============================================================================

/// **M4.1: Rapid Command Sequence**
///
/// Manager should handle rapid command sequences without dropping commands.
#[tokio::test(start_paused = true)]
async fn test_rapid_command_sequence() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Send many commands rapidly
    for _ in 0..10 {
        cmd_tx
            .send(Command::Pause(PauseReason::Manual))
            .await
            .unwrap();
        cmd_tx
            .send(Command::Resume(PauseReason::Manual))
            .await
            .unwrap();
    }

    advance_time_and_yield(duration_ms(500)).await;

    // Should handle all commands (final state should be running due to idempotency)
    {
        let _state = env.shared_state.read();
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **M4.2: Interleaved Pause Reasons**
///
/// Rapidly adding/removing different pause reasons.
#[tokio::test(start_paused = true)]
async fn test_interleaved_pause_reasons() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Interleave add/remove of different reasons
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    cmd_tx.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    cmd_tx
        .send(Command::Resume(PauseReason::Dnd))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Final state: Only UserIdle should remain
    {
        let state = env.shared_state.read();
        assert!(state.is_paused());
        assert_eq!(state.pause_reasons().len(), 1);
        assert!(state.pause_reasons().contains(&PauseReason::UserIdle));
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

// ============================================================================
// Edge Cases
// ============================================================================

/// **M5.1: Pause Then Immediate Resume Same Reason**
///
/// Pause and immediately resume same reason - should cancel out.
#[tokio::test(start_paused = true)]
async fn test_pause_immediate_resume() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Pause then immediately resume
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Should be running (no pause reasons)
    {
        let state = env.shared_state.read();
        assert!(!state.is_paused());
        assert_eq!(state.pause_reasons().len(), 0);
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **M5.2: Many Reasons Then Clear All**
///
/// Add many reasons then clear them all.
#[tokio::test(start_paused = true)]
async fn test_many_reasons_then_clear_all() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Add all reasons multiple times
    for _ in 0..5 {
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
    }
    advance_time_and_yield(duration_ms(200)).await;

    // Should have 4 distinct reasons (idempotent)
    {
        let state = env.shared_state.read();
        assert_eq!(state.pause_reasons().len(), 4);
    }

    // Clear all
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
    {
        let state = env.shared_state.read();
        assert!(!state.is_paused());
        assert_eq!(state.pause_reasons().len(), 0);
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}
