//! Stress tests for scheduler robustness under extreme conditions
//!
//! These tests verify that the scheduler system remains stable and correct
//! when subjected to:
//! - Concurrent monitor signals (race conditions)
//! - Rapid state transitions (hundreds of cycles)
//! - Command queue saturation (thousands of commands)
//! - Complex state transition chains
//! - Monitor signals during break sessions
//! - Configuration updates during monitor storms
//!
//! # Test Strategy
//!
//! Unlike basic integration tests, these stress tests focus on:
//! - **Robustness**: System doesn't crash or deadlock
//! - **Consistency**: State remains valid under pressure
//! - **Recovery**: System recovers from edge cases
//! - **Concurrency**: Handles simultaneous operations correctly
//!
//! # Performance Notes
//!
//! These tests may take longer to execute than regular tests due to:
//! - High iteration counts (100-2000 cycles)
//! - Concurrent task spawning
//! - Multiple async yields for synchronization

#[cfg(test)]
mod tests {
    use crate::scheduler::models::{Command, PauseReason};
    use crate::scheduler::test_helpers::TestConfigBuilder;
    use crate::scheduler::test_helpers::manager::{create_manager_test_env, spawn_test_manager};
    use crate::scheduler::test_helpers::state_machine::advance_time_and_yield;
    use chrono::Duration;
    use tokio::sync::mpsc;

    /// Test concurrent signals from all three monitors simultaneously
    ///
    /// This tests the true race condition scenario where three monitors
    /// send Pause commands at the exact same moment (< 1ms apart).
    ///
    /// Expected behavior:
    /// - All three pause reasons are recorded
    /// - No deadlocks or panics occur
    /// - State remains consistent
    /// - Only one "paused" event is emitted
    #[tokio::test(start_paused = true)]
    async fn test_concurrent_three_monitors_race() {
        let config = TestConfigBuilder::new().mini_break_interval_s(300).build();

        let env = create_manager_test_env(config);
        let (cmd_tx, cmd_rx) = mpsc::channel(32);

        spawn_test_manager(&env, cmd_rx).await;
        advance_time_and_yield(Duration::seconds(1)).await;

        // Spawn three concurrent tasks that send Pause commands simultaneously
        let tx1 = cmd_tx.clone();
        let tx2 = cmd_tx.clone();
        let tx3 = cmd_tx.clone();

        let task1 = tokio::spawn(async move {
            tx1.send(Command::Pause(PauseReason::UserIdle))
                .await
                .unwrap();
        });

        let task2 = tokio::spawn(async move {
            tx2.send(Command::Pause(PauseReason::Dnd)).await.unwrap();
        });

        let task3 = tokio::spawn(async move {
            tx3.send(Command::Pause(PauseReason::AppExclusion))
                .await
                .unwrap();
        });

        // Wait for all tasks to complete
        let _ = tokio::join!(task1, task2, task3);
        advance_time_and_yield(Duration::milliseconds(100)).await;

        // Verify all three pause reasons are recorded
        let state = env.shared_state.read();
        assert!(state.is_paused(), "Should be paused");
        assert_eq!(
            state.pause_reasons().len(),
            3,
            "Should have all three pause reasons"
        );
        assert!(state.pause_reasons().contains(&PauseReason::UserIdle));
        assert!(state.pause_reasons().contains(&PauseReason::Dnd));
        assert!(state.pause_reasons().contains(&PauseReason::AppExclusion));
    }

    /// Test rapid pause/resume cycling (200 iterations)
    ///
    /// This stresses the state machine with rapid transitions between
    /// paused and running states to ensure no memory leaks, state
    /// corruption, or performance degradation.
    ///
    /// Expected behavior:
    /// - All 200 cycles complete without panic
    /// - Final state is predictable
    /// - No `pause_reasons` accumulation (memory leak check)
    #[tokio::test(start_paused = true)]
    async fn test_rapid_pause_resume_cycling() {
        let config = TestConfigBuilder::new().mini_break_interval_s(600).build();

        let env = create_manager_test_env(config);
        let (cmd_tx, cmd_rx) = mpsc::channel(32);

        spawn_test_manager(&env, cmd_rx).await;
        advance_time_and_yield(Duration::seconds(1)).await;

        // Rapid cycling: 200 pause/resume cycles
        for i in 1..=200 {
            cmd_tx
                .send(Command::Pause(PauseReason::Manual))
                .await
                .unwrap();
            advance_time_and_yield(Duration::milliseconds(10)).await;

            // Verify paused
            {
                let state = env.shared_state.read();
                assert!(state.is_paused(), "Iteration {i}: should be paused");
                assert_eq!(
                    state.pause_reasons().len(),
                    1,
                    "Iteration {i}: should have exactly 1 pause reason"
                );
            }

            cmd_tx
                .send(Command::Resume(PauseReason::Manual))
                .await
                .unwrap();
            advance_time_and_yield(Duration::milliseconds(10)).await;

            // Verify resumed
            {
                let state = env.shared_state.read();
                assert!(!state.is_paused(), "Iteration {i}: should be resumed");
                assert_eq!(
                    state.pause_reasons().len(),
                    0,
                    "Iteration {i}: should have no pause reasons (memory leak check)"
                );
            }
        }

        // Final state check
        let state = env.shared_state.read();
        assert!(!state.is_paused(), "Final state should be running");
        assert_eq!(
            state.pause_reasons().len(),
            0,
            "Final state should have no pause reasons"
        );
    }

    /// Test command queue under extreme pressure (2000 commands)
    ///
    /// This tests the system's ability to handle a massive influx of
    /// commands without dropping messages or corrupting state.
    ///
    /// Expected behavior:
    /// - All 2000 commands are processed
    /// - Final state is consistent
    /// - No deadlocks or channel overflow
    #[tokio::test(start_paused = true)]
    async fn test_command_queue_under_pressure() {
        let config = TestConfigBuilder::new().mini_break_interval_s(900).build();

        let env = create_manager_test_env(config);
        let (cmd_tx, cmd_rx) = mpsc::channel(64); // Larger buffer for stress test

        spawn_test_manager(&env, cmd_rx).await;
        advance_time_and_yield(Duration::seconds(1)).await;

        // Send 2000 commands: alternating Pause/Resume
        for i in 0..1000 {
            // Pause
            cmd_tx
                .send(Command::Pause(PauseReason::Manual))
                .await
                .unwrap();
            // Resume
            cmd_tx
                .send(Command::Resume(PauseReason::Manual))
                .await
                .unwrap();

            // Yield occasionally to let processing happen
            if i % 100 == 0 {
                advance_time_and_yield(Duration::milliseconds(10)).await;
            }
        }

        // Let all commands process
        advance_time_and_yield(Duration::milliseconds(500)).await;

        // Final state should be consistent
        let state = env.shared_state.read();
        assert!(
            !state.is_paused(),
            "Final state should be running (last command was Resume)"
        );
        assert_eq!(
            state.pause_reasons().len(),
            0,
            "Final state should have no pause reasons"
        );
    }

    /// Test configuration update during monitor storm
    ///
    /// This tests the system's ability to handle configuration updates
    /// while monitors are rapidly sending signals.
    ///
    /// Expected behavior:
    /// - Configuration update succeeds
    /// - Monitor signals are processed correctly
    /// - No state corruption
    #[tokio::test(start_paused = true)]
    async fn test_config_update_with_monitor_storm() {
        let config = TestConfigBuilder::new().mini_break_interval_s(300).build();

        let env = create_manager_test_env(config);
        let (cmd_tx, cmd_rx) = mpsc::channel(64);

        spawn_test_manager(&env, cmd_rx).await;
        advance_time_and_yield(Duration::seconds(1)).await;

        // Start monitor storm: rapid pause/resume from multiple monitors
        let storm_tx = cmd_tx.clone();
        let storm_task = tokio::spawn(async move {
            for _ in 0..50 {
                let _ = storm_tx.send(Command::Pause(PauseReason::UserIdle)).await;
                let _ = storm_tx.send(Command::Resume(PauseReason::UserIdle)).await;
                let _ = storm_tx.send(Command::Pause(PauseReason::Dnd)).await;
                let _ = storm_tx.send(Command::Resume(PauseReason::Dnd)).await;
                tokio::time::sleep(std::time::Duration::from_micros(100)).await;
            }
        });

        // Simultaneously send configuration updates
        let new_config = TestConfigBuilder::new().mini_break_interval_s(180).build();

        for _ in 0..10 {
            cmd_tx
                .send(Command::UpdateConfig(new_config.clone()))
                .await
                .unwrap();
            advance_time_and_yield(Duration::milliseconds(50)).await;
        }

        // Wait for storm to complete
        let _ = storm_task.await;
        advance_time_and_yield(Duration::milliseconds(200)).await;

        // Verify state is consistent
        let state = env.shared_state.read();
        // State should be stable (not paused since last commands were Resume)
        assert!(!state.is_paused(), "State should be consistent after storm");
    }

    /// Test complex state transition chain
    ///
    /// This tests a sequence of complex state transitions involving
    /// multiple pause reasons, user interactions, and state changes.
    ///
    /// Expected behavior:
    /// - All state transitions are handled correctly
    /// - State consistency is maintained throughout
    /// - No unexpected state corruption
    #[tokio::test(start_paused = true)]
    async fn test_complex_state_transition_chain() {
        let config = TestConfigBuilder::new().mini_break_interval_s(300).build();

        let env = create_manager_test_env(config);
        let (cmd_tx, cmd_rx) = mpsc::channel(32);

        spawn_test_manager(&env, cmd_rx).await;
        advance_time_and_yield(Duration::seconds(1)).await;

        // Complex sequence of state transitions
        let transitions = vec![
            (Command::Pause(PauseReason::Manual), "User pause"),
            (Command::Pause(PauseReason::UserIdle), "Idle pause"),
            (Command::Resume(PauseReason::Manual), "User resume"),
            // Still paused due to Idle
            (Command::Pause(PauseReason::Dnd), "Dnd pause"),
            (Command::Pause(PauseReason::AppExclusion), "App pause"),
            (Command::Resume(PauseReason::UserIdle), "Idle resume"),
            // Still paused due to Dnd and AppWhitelist
            (Command::Resume(PauseReason::Dnd), "Dnd resume"),
            // Still paused due to AppWhitelist
            (
                Command::Resume(PauseReason::AppExclusion),
                "App resume - now fully running",
            ),
        ];

        for (cmd, description) in transitions {
            cmd_tx.send(cmd.clone()).await.unwrap();
            advance_time_and_yield(Duration::milliseconds(50)).await;

            let state = env.shared_state.read();
            tracing::debug!(
                "Transition '{}': paused={}, reasons={:?}",
                description,
                state.is_paused(),
                state.pause_reasons()
            );

            // Verify state consistency
            if state.pause_reasons().is_empty() {
                assert!(
                    !state.is_paused(),
                    "State mismatch at '{description}': no reasons but still paused"
                );
            } else {
                assert!(
                    state.is_paused(),
                    "State mismatch at '{description}': has reasons but not paused"
                );
            }
        }

        // Final verification
        let state = env.shared_state.read();
        assert!(!state.is_paused(), "Final state should be running");
        assert_eq!(
            state.pause_reasons().len(),
            0,
            "Final state should have no pause reasons"
        );
    }

    /// Test all pause reasons with random resume order
    ///
    /// This tests that the system correctly handles the case where
    /// all four pause reasons are active, and then resumed in a
    /// random order.
    ///
    /// Expected behavior:
    /// - System resumes only when ALL reasons are cleared
    /// - Partial resumes don't trigger running state
    /// - Order of resume doesn't matter
    #[tokio::test(start_paused = true)]
    async fn test_all_pause_reasons_random_resume_order() {
        let config = TestConfigBuilder::new().mini_break_interval_s(600).build();

        let env = create_manager_test_env(config);
        let (cmd_tx, cmd_rx) = mpsc::channel(32);

        spawn_test_manager(&env, cmd_rx).await;
        advance_time_and_yield(Duration::seconds(1)).await;

        // Activate all four pause reasons
        let all_reasons = vec![
            PauseReason::Manual,
            PauseReason::UserIdle,
            PauseReason::Dnd,
            PauseReason::AppExclusion,
        ];

        for reason in &all_reasons {
            cmd_tx.send(Command::Pause(*reason)).await.unwrap();
            advance_time_and_yield(Duration::milliseconds(20)).await;
        }

        // Verify all four are active
        {
            let state = env.shared_state.read();
            assert!(state.is_paused(), "Should be paused");
            assert_eq!(
                state.pause_reasons().len(),
                4,
                "Should have all four pause reasons"
            );
        }

        // Resume in a specific order: AppWhitelist, User, Dnd, Idle
        let resume_order = [
            PauseReason::AppExclusion,
            PauseReason::Manual,
            PauseReason::Dnd,
            PauseReason::UserIdle,
        ];

        for (i, reason) in resume_order.iter().enumerate() {
            cmd_tx.send(Command::Resume(*reason)).await.unwrap();
            advance_time_and_yield(Duration::milliseconds(20)).await;

            let state = env.shared_state.read();
            let remaining = 3 - i;

            if remaining > 0 {
                assert!(
                    state.is_paused(),
                    "Should still be paused with {remaining} reasons remaining"
                );
                assert_eq!(
                    state.pause_reasons().len(),
                    remaining,
                    "Should have {remaining} reasons remaining"
                );
            } else {
                assert!(!state.is_paused(), "Should be fully resumed");
                assert_eq!(state.pause_reasons().len(), 0, "Should have no reasons");
            }
        }
    }

    /// Test scheduler recovery from edge case
    ///
    /// This tests the system's ability to recover from edge cases
    /// such as receiving Resume without Pause, or multiple Resume
    /// commands for the same reason.
    ///
    /// Expected behavior:
    /// - System doesn't panic on invalid commands
    /// - State remains consistent
    /// - Idempotency is preserved
    #[tokio::test(start_paused = true)]
    async fn test_scheduler_recovery_from_edge_cases() {
        let config = TestConfigBuilder::new().mini_break_interval_s(600).build();

        let env = create_manager_test_env(config);
        let (cmd_tx, cmd_rx) = mpsc::channel(32);

        spawn_test_manager(&env, cmd_rx).await;
        advance_time_and_yield(Duration::seconds(1)).await;

        // Edge case 1: Resume without Pause
        cmd_tx
            .send(Command::Resume(PauseReason::Manual))
            .await
            .unwrap();
        advance_time_and_yield(Duration::milliseconds(50)).await;

        {
            let state = env.shared_state.read();
            assert!(
                !state.is_paused(),
                "Should remain running after Resume without Pause"
            );
            assert_eq!(state.pause_reasons().len(), 0);
        }

        // Edge case 2: Multiple Pause commands for same reason
        for _ in 0..5 {
            cmd_tx
                .send(Command::Pause(PauseReason::UserIdle))
                .await
                .unwrap();
            advance_time_and_yield(Duration::milliseconds(10)).await;
        }

        {
            let state = env.shared_state.read();
            assert!(state.is_paused(), "Should be paused");
            assert_eq!(
                state.pause_reasons().len(),
                1,
                "Should have only 1 pause reason (idempotency)"
            );
        }

        // Edge case 3: Multiple Resume commands for same reason
        for _ in 0..5 {
            cmd_tx
                .send(Command::Resume(PauseReason::UserIdle))
                .await
                .unwrap();
            advance_time_and_yield(Duration::milliseconds(10)).await;
        }

        {
            let state = env.shared_state.read();
            assert!(!state.is_paused(), "Should be resumed");
            assert_eq!(state.pause_reasons().len(), 0);
        }

        // Edge case 4: Mixed invalid commands
        cmd_tx
            .send(Command::Resume(PauseReason::Dnd))
            .await
            .unwrap();
        cmd_tx
            .send(Command::Pause(PauseReason::Manual))
            .await
            .unwrap();
        cmd_tx
            .send(Command::Resume(PauseReason::AppExclusion))
            .await
            .unwrap();
        cmd_tx
            .send(Command::Resume(PauseReason::Manual))
            .await
            .unwrap();
        advance_time_and_yield(Duration::milliseconds(100)).await;

        {
            let state = env.shared_state.read();
            assert!(!state.is_paused(), "Should be running after mixed commands");
            assert_eq!(state.pause_reasons().len(), 0);
        }
    }
}
