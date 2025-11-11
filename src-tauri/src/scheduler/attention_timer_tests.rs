//! Integration tests for `AttentionTimer`
//!
//! Tests the attention reminder system which provides periodic
//! attention prompts at specific times of day.
//!
//! # Test Coverage
//!
//! - **Basic Scheduling**: Attention events at configured times
//! - **Time Range Validation**: Day boundaries, time range checks
//! - **Pause/Resume**: Attention timer respects pause states
//! - **Configuration Updates**: Runtime config changes
//! - **Edge Cases**: Past times, multiple attentions, midnight crossing
//!
//! TODO: Add methods to verify that attention events were actually emitted.

use tokio::sync::mpsc;

use crate::config::AppConfig;
use crate::core::schedule::{AttentionId, AttentionSettings};
use crate::core::theme::ThemeSettings;
use crate::core::time::ShortTimes;
use crate::scheduler::models::{Command, PauseReason};
use crate::scheduler::test_helpers::manager::*;
use crate::scheduler::test_helpers::state_machine::advance_time_and_yield;
use crate::scheduler::test_helpers::*;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a config with attention settings
fn config_with_attention(
    hour: u32,
    minute: u32,
    enabled: bool,
    days: Vec<chrono::Weekday>,
) -> AppConfig {
    let mut config = TestConfigBuilder::new().build();

    config.attentions = vec![AttentionSettings {
        id: AttentionId::new(),
        name: "Test Attention".to_string(),
        enabled,
        theme: ThemeSettings::default(),
        times: ShortTimes::new(vec![naive_time(hour, minute, 0)]),
        days_of_week: days,
        title: "Attention".to_string(),
        message: "Time for attention".to_string(),
        duration_s: 10,
    }];

    config
}

// ============================================================================
// Basic Attention Tests
// ============================================================================

/// **ATT1.1: Attention Timer Starts**
///
/// `AttentionTimer` should start and be ready to schedule.
#[tokio::test(start_paused = true)]
async fn test_attention_timer_starts() {
    let config = config_with_attention(10, 0, true, all_weekdays());

    let env = create_manager_test_env(config);
    let (_cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Timer should start without errors
    // (No direct way to verify in test, but it shouldn't crash)

    // Cleanup
    drop(env.shutdown_tx);
}

/// **ATT1.2: Attention Timer With No Attentions**
///
/// Timer should handle empty attention list gracefully.
#[tokio::test(start_paused = true)]
async fn test_attention_timer_no_attentions() {
    let mut config = TestConfigBuilder::new().build();
    config.attentions = vec![]; // No attentions configured

    let env = create_manager_test_env(config);
    let (_cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Should not crash with empty attention list
    advance_time_and_yield(duration_s(10)).await;

    // Cleanup
    drop(env.shutdown_tx);
}

/// **ATT1.3: Disabled Attention**
///
/// Disabled attentions should not trigger.
#[tokio::test(start_paused = true)]
async fn test_disabled_attention() {
    let config = config_with_attention(10, 0, false, all_weekdays()); // Disabled

    let env = create_manager_test_env(config);
    let (_cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Wait some time - should not trigger
    advance_time_and_yield(duration_s(5)).await;

    // No way to verify event wasn't sent in this test setup
    // But timer should handle it gracefully

    // Cleanup
    drop(env.shutdown_tx);
}

// ============================================================================
// Pause/Resume Tests
// ============================================================================

/// **ATT2.1: Pause Stops Attention Timer**
///
/// When scheduler is paused, attention timer should also pause.
#[tokio::test(start_paused = true)]
async fn test_pause_stops_attention_timer() {
    let config = config_with_attention(10, 0, true, all_weekdays());

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Pause scheduler
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify paused
    {
        let state = env.shared_state.read();
        assert!(state.is_paused());
    }

    // Wait some time - attention should not trigger while paused
    advance_time_and_yield(duration_s(5)).await;

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **ATT2.2: Resume Restarts Attention Timer**
///
/// When scheduler resumes, attention timer should resume.
#[tokio::test(start_paused = true)]
async fn test_resume_restarts_attention_timer() {
    let config = config_with_attention(10, 0, true, all_weekdays());

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Pause
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Resume
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify resumed
    {
        let state = env.shared_state.read();
        assert!(!state.is_paused());
    }

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **ATT2.3: Multiple Pause Reasons**
///
/// Attention timer should respect multi-reason pause management.
#[tokio::test(start_paused = true)]
async fn test_attention_multiple_pause_reasons() {
    let config = config_with_attention(10, 0, true, all_weekdays());

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

    // Remove one
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // Should still be paused
    assert!(env.shared_state.read().is_paused());

    // Remove last
    cmd_tx
        .send(Command::Resume(PauseReason::Dnd))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    // Should be running
    assert!(!env.shared_state.read().is_paused());

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

// ============================================================================
// Configuration Update Tests
// ============================================================================

/// **ATT3.1: Update Attention Config**
///
/// Attention timer should handle config updates.
#[tokio::test(start_paused = true)]
async fn test_update_attention_config() {
    let config = config_with_attention(10, 0, true, all_weekdays());

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Update config with different attention time
    let new_config = config_with_attention(14, 0, true, all_weekdays());
    cmd_tx
        .send(Command::UpdateConfig(new_config))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Should handle update without crashing
    advance_time_and_yield(duration_s(1)).await;

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **ATT3.2: Add New Attention**
///
/// Adding a new attention to config.
#[tokio::test(start_paused = true)]
async fn test_add_new_attention() {
    let config = config_with_attention(10, 0, true, all_weekdays());

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Update with multiple attentions
    let mut new_config = config_with_attention(10, 0, true, all_weekdays());
    new_config.attentions.push(AttentionSettings {
        id: AttentionId::new(),
        name: "Second Attention".to_string(),
        enabled: true,
        theme: ThemeSettings::default(),
        times: ShortTimes::new(vec![naive_time(14, 0, 0)]),
        days_of_week: all_weekdays(),
        title: "Attention".to_string(),
        message: "Time for attention".to_string(),
        duration_s: 10,
    });

    cmd_tx
        .send(Command::UpdateConfig(new_config))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Should handle multiple attentions
    advance_time_and_yield(duration_s(1)).await;

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **ATT3.3: Remove All Attentions**
///
/// Removing all attentions should be handled gracefully.
#[tokio::test(start_paused = true)]
async fn test_remove_all_attentions() {
    let config = config_with_attention(10, 0, true, all_weekdays());

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Update with empty attentions
    let mut new_config = TestConfigBuilder::new().build();
    new_config.attentions = vec![];

    cmd_tx
        .send(Command::UpdateConfig(new_config))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Should handle empty list
    advance_time_and_yield(duration_s(1)).await;

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

// ============================================================================
// Multiple Attentions Tests
// ============================================================================

/// **ATT4.1: Multiple Attentions Different Times**
///
/// Multiple attentions at different times should be scheduled correctly.
#[tokio::test(start_paused = true)]
async fn test_multiple_attentions_different_times() {
    let mut config = TestConfigBuilder::new().build();

    config.attentions = vec![
        AttentionSettings {
            id: AttentionId::new(),
            name: "Morning Attention".to_string(),
            enabled: true,
            theme: ThemeSettings::default(),
            times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
            days_of_week: all_weekdays(),
            title: "Attention".to_string(),
            message: "Time for attention".to_string(),
            duration_s: 10,
        },
        AttentionSettings {
            id: AttentionId::new(),
            name: "Afternoon Attention".to_string(),
            enabled: true,
            theme: ThemeSettings::default(),
            times: ShortTimes::new(vec![naive_time(14, 0, 0)]),
            days_of_week: all_weekdays(),
            title: "Attention".to_string(),
            message: "Time for attention".to_string(),
            duration_s: 10,
        },
        AttentionSettings {
            id: AttentionId::new(),
            name: "Evening Attention".to_string(),
            enabled: true,
            theme: ThemeSettings::default(),
            times: ShortTimes::new(vec![naive_time(16, 0, 0)]),
            days_of_week: all_weekdays(),
            title: "Attention".to_string(),
            message: "Time for attention".to_string(),
            duration_s: 10,
        },
    ];

    let env = create_manager_test_env(config);
    let (_cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Should pick the next closest attention
    advance_time_and_yield(duration_s(1)).await;

    // Cleanup
    drop(env.shutdown_tx);
}

// ============================================================================
// Days of Week Tests
// ============================================================================

/// **ATT5.1: Attention On Specific Days**
///
/// Attention should only trigger on configured days.
#[tokio::test(start_paused = true)]
async fn test_attention_specific_days() {
    // Only on Monday and Wednesday
    let config = config_with_attention(
        10,
        0,
        true,
        vec![chrono::Weekday::Mon, chrono::Weekday::Wed],
    );

    let env = create_manager_test_env(config);
    let (_cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Should calculate next valid day
    advance_time_and_yield(duration_s(1)).await;

    // Cleanup
    drop(env.shutdown_tx);
}

/// **ATT5.2: Attention Weekdays Only**
///
/// Attention configured for weekdays only.
#[tokio::test(start_paused = true)]
async fn test_attention_weekdays_only() {
    let config = config_with_attention(10, 0, true, workdays());

    let env = create_manager_test_env(config);
    let (_cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Should skip weekends
    advance_time_and_yield(duration_s(1)).await;

    // Cleanup
    drop(env.shutdown_tx);
}

/// **ATT5.3: Attention Weekend Only**
///
/// Attention configured for weekend only.
#[tokio::test(start_paused = true)]
async fn test_attention_weekend_only() {
    let config = config_with_attention(10, 0, true, weekend_days());

    let env = create_manager_test_env(config);
    let (_cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Should skip weekdays
    advance_time_and_yield(duration_s(1)).await;

    // Cleanup
    drop(env.shutdown_tx);
}

// ============================================================================
// Edge Cases
// ============================================================================

/// **ATT6.1: Attention Time Already Passed Today**
///
/// If attention time already passed today, should schedule for next valid day.
#[tokio::test(start_paused = true)]
async fn test_attention_time_passed() {
    // Set attention for early morning (already passed in test)
    let config = config_with_attention(1, 0, true, all_weekdays());

    let env = create_manager_test_env(config);
    let (_cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Should calculate next occurrence (tomorrow or next valid day)
    advance_time_and_yield(duration_s(1)).await;

    // Cleanup
    drop(env.shutdown_tx);
}

/// **ATT6.2: Attention At Midnight**
///
/// Attention scheduled at midnight (00:00).
#[tokio::test(start_paused = true)]
async fn test_attention_at_midnight() {
    let config = config_with_attention(0, 0, true, all_weekdays());

    let env = create_manager_test_env(config);
    let (_cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Should handle midnight correctly
    advance_time_and_yield(duration_s(1)).await;

    // Cleanup
    drop(env.shutdown_tx);
}

/// **ATT6.3: Attention At 23:59**
///
/// Attention scheduled at end of day.
#[tokio::test(start_paused = true)]
async fn test_attention_at_end_of_day() {
    let config = config_with_attention(23, 59, true, all_weekdays());

    let env = create_manager_test_env(config);
    let (_cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Should handle end of day correctly
    advance_time_and_yield(duration_s(1)).await;

    // Cleanup
    drop(env.shutdown_tx);
}

/// **ATT6.4: No Valid Days Configured**
///
/// Attention with empty `days_of_week` list.
#[tokio::test(start_paused = true)]
async fn test_attention_no_valid_days() {
    let config = config_with_attention(10, 0, true, vec![]); // No days

    let env = create_manager_test_env(config);
    let (_cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Should handle gracefully (never trigger)
    advance_time_and_yield(duration_s(1)).await;

    // Cleanup
    drop(env.shutdown_tx);
}

// ============================================================================
// Integration Tests
// ============================================================================

/// **ATT7.1: Attention With Breaks**
///
/// Attention timer should coexist with break scheduler.
#[tokio::test(start_paused = true)]
async fn test_attention_with_breaks() {
    let mut config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    config.attentions = vec![AttentionSettings {
        id: AttentionId::new(),
        name: "Test Attention".to_string(),
        enabled: true,
        theme: ThemeSettings::default(),
        times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
        days_of_week: all_weekdays(),
        title: "Attention".to_string(),
        message: "Time for attention".to_string(),
        duration_s: 10,
    }];

    let env = create_manager_test_env(config);
    let (_cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Both break scheduler and attention timer should be running
    advance_time_and_yield(duration_s(5)).await;

    // Cleanup
    drop(env.shutdown_tx);
}

/// **ATT7.2: Pause Affects Both Break and Attention**
///
/// Pausing should affect both schedulers.
#[tokio::test(start_paused = true)]
async fn test_pause_affects_both_schedulers() {
    let mut config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    config.attentions = vec![AttentionSettings {
        id: AttentionId::new(),
        name: "Test Attention".to_string(),
        enabled: true,
        theme: ThemeSettings::default(),
        times: ShortTimes::new(vec![naive_time(10, 0, 0)]),
        days_of_week: all_weekdays(),
        title: "Attention".to_string(),
        message: "Time for attention".to_string(),
        duration_s: 10,
    }];

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Pause
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    assert!(env.shared_state.read().is_paused());

    // Resume
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    assert!(!env.shared_state.read().is_paused());

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}
