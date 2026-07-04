//! Comprehensive behavior tests for `BreakScheduler`
//!
//! These tests verify the scheduler's behavior across normal operations,
//! edge cases, and error scenarios.
//!
//! # Test Categories
//!
//! - **Basic Flow**: Normal operation lifecycle
//! - **Time Scheduling**: Work hours, notifications, time boundaries
//! - **User Interactions**: Pause, postpone, skip, manual trigger
//! - **Configuration**: Config updates, validation
//! - **Edge Cases**: Boundary conditions, extreme values
//! - **Error Scenarios**: Window failures, missing events, system time changes

use tokio::sync::mpsc;

use crate::core::break_kind::BreakKind;
use crate::core::schedule::DaysOfWeek;
use crate::scheduler::models::{Command, PauseReason, SchedulerEvent};
use crate::scheduler::test_helpers::state_machine::*;
use crate::scheduler::test_helpers::*;
use crate::{
    core::schedule::BreakId,
    scheduler::test_helpers::manager::{create_manager_test_env, spawn_test_manager},
};

// ============================================================================
// Section 1: Basic Flow Tests
// ============================================================================

/// **T1.1: Application Startup - Automatic Initialization**
///
/// The scheduler should automatically start scheduling breaks after startup.
#[tokio::test(start_paused = true)]
async fn automatic_startup() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(1200) // 20 minutes
        .mini_break_duration_s(300) // 5 minutes
        .long_break_after_mini_breaks(4)
        .build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    // Give scheduler time to initialize
    advance_time_and_yield(duration_ms(200)).await;

    // Verify scheduler automatically started
    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    let status = get_latest_status(&emitter);
    assert!(
        !status.paused,
        "Scheduler should NOT be paused after startup"
    );
    assert!(
        status.next_event.is_some(),
        "Scheduler should automatically schedule first break"
    );

    if let Some(event_info) = status.next_event {
        assert!(
            matches!(event_info.kind, SchedulerEvent::MiniBreak(_)),
            "First break should be mini break"
        );
        assert_duration_near(event_info.seconds_until.into(), 1200, 5);
    }

    assert_eq!(status.mini_break_counter, 0);

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **T1.2: Complete Break Cycle**
///
/// Verifies a full break cycle from scheduling to completion.
#[tokio::test(start_paused = true)]
async fn complete_break_cycle() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(60)
        .mini_break_duration_s(20)
        .notification_before_s(0)
        .build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    // Wait for break to trigger
    advance_time_and_yield(duration_s(60)).await;

    assert!(emitter.has_event("scheduler-event"), "Break should trigger");

    let break_events = emitter.get_events_by_name("scheduler-event");
    let break_event: SchedulerEvent =
        serde_json::from_value(break_events[0].clone()).expect("Should parse");

    assert!(matches!(break_event, SchedulerEvent::MiniBreak(_)));

    emitter.clear();

    // Complete the break
    cmd_tx
        .send(Command::PromptFinished(break_event))
        .await
        .unwrap();
    advance_time_and_yield(duration_s(1)).await;

    // Verify next break scheduled
    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    let status = get_latest_status(&emitter);
    assert_eq!(status.mini_break_counter, 1);

    let next_event = status.next_event.unwrap();
    assert!(matches!(next_event.kind, SchedulerEvent::MiniBreak(_)));
    assert_duration_near(next_event.seconds_until.into(), 60, 1);

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **T1.3: Long Break Trigger**
///
/// Verifies long break is triggered after N mini breaks and counter resets.
#[tokio::test(start_paused = true)]
async fn long_break_trigger() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(30)
        .mini_break_duration_s(10)
        .long_break_after_mini_breaks(3)
        .long_break_duration_s(20)
        .notification_before_s(0)
        .build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    // Complete 3 mini breaks
    for i in 1..=3 {
        advance_time_and_yield(duration_s(30)).await;

        let break_events = emitter.get_events_by_name("scheduler-event");
        let break_event: SchedulerEvent =
            serde_json::from_value(break_events[0].clone()).expect("Should parse");

        assert!(
            matches!(break_event, SchedulerEvent::MiniBreak(_)),
            "Break {i} should be mini",
        );

        emitter.clear();
        cmd_tx
            .send(Command::PromptFinished(break_event))
            .await
            .unwrap();
        advance_time_and_yield(duration_s(1)).await;

        cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
        advance_time_and_yield(duration_ms(200)).await;
        let status = get_latest_status(&emitter);
        assert_eq!(status.mini_break_counter, i);
        emitter.clear();
    }

    // 4th break should be long
    advance_time_and_yield(duration_s(30)).await;

    let break_events = emitter.get_events_by_name("scheduler-event");
    let break_event: SchedulerEvent =
        serde_json::from_value(break_events[0].clone()).expect("Should parse");

    assert!(
        matches!(break_event, SchedulerEvent::LongBreak(_)),
        "4th break should be long"
    );

    emitter.clear();
    cmd_tx
        .send(Command::PromptFinished(break_event))
        .await
        .unwrap();
    advance_time_and_yield(duration_s(1)).await;

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    let status = get_latest_status(&emitter);
    assert_eq!(status.mini_break_counter, 0, "Counter should reset");

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

// ============================================================================
// Section 2: User Interactions
// ============================================================================

/// **T2.1: Manual Pause and Resume**
///
/// User can manually pause/resume via tray menu.
#[tokio::test(start_paused = true)]
async fn manual_pause_resume() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    // Verify initially running
    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    let status = get_latest_status(&emitter);
    assert!(!status.paused);

    emitter.clear();

    // User pauses
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    let status = get_latest_status(&emitter);
    assert!(status.paused, "Should be paused");

    emitter.clear();

    // User resumes
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    let status = get_latest_status(&emitter);
    assert!(!status.paused, "Should be running");

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **T2.2: Postpone Break**
///
/// User can postpone upcoming break.
#[tokio::test(start_paused = true)]
async fn postpone_break() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(60)
        .notification_before_s(0)
        .postpone_settings(2, 300)
        .build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    // Wait close to break time
    advance_time_and_yield(duration_s(55)).await;

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    let status_before = get_latest_status(&emitter);
    let seconds_before = status_before.next_event.unwrap().seconds_until;

    emitter.clear();

    // Postpone
    cmd_tx.send(Command::PostponeBreak).await.unwrap();
    advance_time_and_yield(duration_s(1)).await;

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    let status_after = get_latest_status(&emitter);
    let seconds_after = status_after.next_event.unwrap().seconds_until;

    assert_duration_near(seconds_after.into(), i64::from(seconds_before) + 300, 5);

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **T2.3: Postpone Limit**
///
/// Verify postpone limit is enforced.
#[tokio::test(start_paused = true)]
async fn postpone_limit() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(60)
        .postpone_settings(2, 30)
        .build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    // Wait for on break
    advance_time_and_yield(duration_s(65)).await;
    emitter.clear();

    // Postpone once to get baseline (this transitions from InBreak to WaitingForBreak)
    cmd_tx.send(Command::PostponeBreak).await.unwrap();
    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    let status_before = get_latest_status(&emitter);
    let seconds_before = status_before.next_event.unwrap().seconds_until;

    emitter.clear();

    // Postpone once more (limit reached)
    cmd_tx.send(Command::PostponeBreak).await.unwrap();
    emitter.clear();

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    let status = get_latest_status(&emitter);
    let seconds_after = status.next_event.unwrap().seconds_until;
    assert_duration_near(seconds_after.into(), i64::from(seconds_before) + 30, 1);

    emitter.clear();

    // Third should fail
    cmd_tx.send(Command::PostponeBreak).await.unwrap();
    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    let status_after = get_latest_status(&emitter);
    let seconds_after = status_after.next_event.unwrap().seconds_until;
    assert_duration_near(seconds_after.into(), i64::from(seconds_before) + 30, 1);

    assert!(emitter.has_event("postpone-limit-reached"));

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **T2.4: Skip Break**
///
/// User can skip upcoming break.
#[tokio::test(start_paused = true)]
async fn skip_break() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(60)
        .notification_before_s(0)
        .build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    // Wait partway
    advance_time_and_yield(duration_s(30)).await;

    // Skip
    cmd_tx.send(Command::SkipBreak).await.unwrap();
    advance_time_and_yield(duration_s(1)).await;

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    let status = get_latest_status(&emitter);
    let seconds_after = status.next_event.unwrap().seconds_until;
    assert_duration_near(seconds_after.into(), 60, 5);

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **T2.5: Manual Trigger**
///
/// User can manually trigger a break from Advanced Option panel.
#[tokio::test(start_paused = true)]
async fn manual_trigger() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(600)
        .notification_before_s(0)
        .build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    // Trigger manually
    cmd_tx
        .send(Command::TriggerEvent(SchedulerEvent::MiniBreak(
            BreakId::new(),
        )))
        .await
        .unwrap();
    advance_time_and_yield(duration_s(1)).await;

    // Should process command
    let event = emitter.get_events_by_name("scheduler-event");
    let break_event: SchedulerEvent =
        serde_json::from_value(event[0].clone()).expect("Should parse");
    assert!(matches!(break_event, SchedulerEvent::MiniBreak(_)));

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **T2.6: Manual Trigger Now Uses Active Schedule**
///
/// User-facing manual triggers resolve the active schedule before executing.
#[tokio::test(start_paused = true)]
async fn manual_trigger_now_uses_active_schedule() {
    let mut first_schedule = TestConfigBuilder::new().build().schedules.remove(0);
    first_schedule.name = "First Enabled But Inactive".to_owned();
    first_schedule.days_of_week = DaysOfWeek::empty();

    let mut active_schedule = TestConfigBuilder::new().build().schedules.remove(0);
    active_schedule.name = "Active Schedule".to_owned();
    active_schedule.days_of_week = DaysOfWeek::all();
    active_schedule.time_range = full_time_range();
    active_schedule.mini_breaks.base.enabled = false;
    let expected_id = active_schedule.mini_breaks.base.id;

    let mut config = TestConfigBuilder::new().build();
    config.schedules = vec![first_schedule, active_schedule];

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    cmd_tx
        .send(Command::TriggerBreakNow(BreakKind::Mini))
        .await
        .unwrap();
    advance_time_and_yield(duration_s(1)).await;

    let events = emitter.get_events_by_name("scheduler-event");
    let event: SchedulerEvent = serde_json::from_value(events[0].clone()).expect("Should parse");
    assert_eq!(event, SchedulerEvent::MiniBreak(expected_id));

    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **T2.7: Manual Trigger Now Falls Back To First Enabled Schedule**
///
/// If no schedule is active, user-facing manual triggers use the first enabled schedule.
#[tokio::test(start_paused = true)]
async fn manual_trigger_now_falls_back_to_first_enabled_schedule() {
    let mut first_enabled = TestConfigBuilder::new().build().schedules.remove(0);
    first_enabled.name = "First Enabled".to_owned();
    first_enabled.days_of_week = DaysOfWeek::empty();
    let expected_id = first_enabled.long_breaks.base.id;

    let mut disabled_active = TestConfigBuilder::new().build().schedules.remove(0);
    disabled_active.name = "Disabled Active".to_owned();
    disabled_active.enabled = false;
    disabled_active.days_of_week = DaysOfWeek::all();
    disabled_active.time_range = full_time_range();

    let mut config = TestConfigBuilder::new().build();
    config.schedules = vec![first_enabled, disabled_active];

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    cmd_tx
        .send(Command::TriggerBreakNow(BreakKind::Long))
        .await
        .unwrap();
    advance_time_and_yield(duration_s(1)).await;

    let events = emitter.get_events_by_name("scheduler-event");
    let event: SchedulerEvent = serde_json::from_value(events[0].clone()).expect("Should parse");
    assert_eq!(event, SchedulerEvent::LongBreak(expected_id));

    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **T2.8: Manual Trigger Will Be Rejected While Paused**
#[tokio::test(start_paused = true)]
async fn manual_trigger_now_rejected_while_paused() {
    let config = TestConfigBuilder::new().build();
    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);
    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    cmd_tx
        .send(Command::TriggerBreakNow(BreakKind::Mini))
        .await
        .unwrap();
    advance_time_and_yield(duration_s(1)).await;

    assert!(emitter.get_events_by_name("scheduler-event").is_empty());

    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **T2.9: Idle Pause Does Not Reset Counter By Default**
#[tokio::test(start_paused = true)]
async fn user_idle_pause_keeps_mini_break_counter_by_default() {
    let config = TestConfigBuilder::new().notification_before_s(0).build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    cmd_tx
        .send(Command::TriggerBreakNow(BreakKind::Mini))
        .await
        .unwrap();
    advance_time_and_yield(duration_s(1)).await;

    let events = emitter.get_events_by_name("scheduler-event");
    let event: SchedulerEvent = serde_json::from_value(events[0].clone()).expect("Should parse");
    cmd_tx.send(Command::PromptFinished(event)).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    let status = get_latest_status(&emitter);
    assert_eq!(status.mini_break_counter, 1);

    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **T2.9: Configured Idle Pause Resets Counter**
#[tokio::test(start_paused = true)]
async fn configured_user_idle_pause_resets_mini_break_counter() {
    let mut config = TestConfigBuilder::new().notification_before_s(0).build();
    config.advanced.reset_mini_break_counter_on_pause_reasons = vec![PauseReason::UserIdle];

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    cmd_tx
        .send(Command::TriggerBreakNow(BreakKind::Mini))
        .await
        .unwrap();
    advance_time_and_yield(duration_s(1)).await;

    let events = emitter.get_events_by_name("scheduler-event");
    let event: SchedulerEvent = serde_json::from_value(events[0].clone()).expect("Should parse");
    cmd_tx.send(Command::PromptFinished(event)).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    cmd_tx
        .send(Command::Pause(PauseReason::UserIdle))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    let status = get_latest_status(&emitter);
    assert_eq!(status.mini_break_counter, 0);

    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

// ============================================================================
// Section 3: Configuration Updates
// ============================================================================

/// **T3.1: Configuration Update**
///
/// Configuration changes should take effect immediately.
#[tokio::test(start_paused = true)]
async fn config_update() {
    let initial = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(initial);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;

    let status_before = get_latest_status(&emitter);
    assert_duration_near(
        status_before.next_event.unwrap().seconds_until.into(),
        60,
        5,
    );

    advance_time_and_yield(duration_s(10)).await;

    // Update config
    let new_config = TestConfigBuilder::new().mini_break_interval_s(120).build();

    cmd_tx
        .send(Command::UpdateConfig(new_config))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    let status_after = get_latest_status(&emitter);
    assert_duration_near(
        status_after.next_event.unwrap().seconds_until.into(),
        120,
        5,
    );

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **T3.2: Request Status**
///
/// `RequestBreakStatus` should emit current state without side effects.
#[tokio::test(start_paused = true)]
async fn request_status() {
    let config = TestConfigBuilder::new().mini_break_interval_s(180).build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    assert!(emitter.has_event("scheduler-status"));

    let events = emitter.get_events_by_name("scheduler-status");
    assert!(!events.is_empty());

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

// ============================================================================
// Section 4: Edge Cases and Error Scenarios
// ============================================================================

/// **E1: Commands During `InBreak` State**
///
/// Verify scheduler handles commands correctly while in break.
#[tokio::test(start_paused = true)]
async fn commands_during_break() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(30)
        .mini_break_duration_s(10)
        .notification_before_s(0)
        .postpone_settings(2, 20)
        .build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    // Trigger break
    advance_time_and_yield(duration_s(35)).await;
    emitter.clear();

    // Try postpone during break (should close window and reschedule)
    cmd_tx.send(Command::PostponeBreak).await.unwrap();
    advance_time_and_yield(duration_s(1)).await;

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    let status = get_latest_status(&emitter);
    assert_duration_near(status.next_event.unwrap().seconds_until.into(), 20, 1);

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **E2: Extreme Configuration - Very Short Interval**
///
/// Verify scheduler handles extremely short intervals.
#[tokio::test(start_paused = true)]
async fn extreme_short_interval() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(5)
        .mini_break_duration_s(2)
        .notification_before_s(0)
        .build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    // Should schedule break in 5 seconds
    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    let status = get_latest_status(&emitter);
    assert_duration_near(status.next_event.unwrap().seconds_until.into(), 5, 1);

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **E3: Multiple Pause Reasons**
///
/// Note: Multiple pause reason coordination is handled by `SchedulerManager`,
/// not by individual `BreakScheduler` instances. `BreakScheduler` itself only
/// tracks internal state transitions.
///
/// This test verifies that `BreakScheduler` correctly responds to Pause/Resume
/// commands from its perspective. The logic of "pause only when first reason
/// added" and "resume only when last reason removed" is in `SchedulerManager`
/// and tested in manager tests.
#[tokio::test(start_paused = true)]
async fn pause_resume_cycle() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    // Pause
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    let status = get_latest_status(&emitter);
    assert!(status.paused, "Should be paused");

    emitter.clear();

    // Resume
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(100)).await;

    cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    advance_time_and_yield(duration_ms(200)).await;
    let status = get_latest_status(&emitter);
    assert!(!status.paused, "Should resume");

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

/// **E4: Rapid Command Sequence**
///
/// Verify scheduler handles rapid command sequences without crashing.
#[tokio::test(start_paused = true)]
async fn rapid_commands() {
    let config = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;
    emitter.clear();

    // Send rapid commands
    for _ in 0..10 {
        cmd_tx.send(Command::RequestBreakStatus).await.unwrap();
    }

    advance_time_and_yield(duration_ms(500)).await;

    // Should handle all commands without crashing
    assert!(emitter.has_event("scheduler-status"));

    let status = get_latest_status(&emitter);
    assert_duration_near(status.next_event.unwrap().seconds_until.into(), 60, 1);

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}

// ============================================================================
// Section 5: State Transition Tests
// ============================================================================

/// **ST1: `InBreak` + Pause(Manual) - Session Must Be Cleaned**
#[tokio::test(start_paused = true)]
async fn inbreak_pause_manual_cleans_session() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(60)
        .mini_break_duration_s(20)
        .build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Wait for break to trigger
    advance_time_and_yield(duration_s(60)).await;

    // Verify session started
    assert!(
        env.shared_state.read().in_any_session(),
        "Session should be active during break"
    );

    // User manually pauses during break
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(500)).await;

    // Verify session cleaned
    assert!(
        !env.shared_state.read().in_any_session(),
        "Session MUST be cleaned when pausing during break"
    );

    // Verify state changed to paused
    assert!(
        env.shared_state.read().is_paused(),
        "Should be in paused state"
    );

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **ST2: `InBreak` + Skip - Session Must Be Cleaned**
#[tokio::test(start_paused = true)]
async fn inbreak_skip_cleans_session() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(60)
        .mini_break_duration_s(20)
        .build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Trigger break
    advance_time_and_yield(duration_s(60)).await;
    assert!(env.shared_state.read().in_any_session());

    // User skips break
    cmd_tx.send(Command::SkipBreak).await.unwrap();
    advance_time_and_yield(duration_ms(500)).await;

    // Session must be cleaned
    assert!(
        !env.shared_state.read().in_any_session(),
        "Session must be cleaned when skipping break"
    );

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **ST3: `InBreak` + Postpone - Session Must Be Cleaned**
#[tokio::test(start_paused = true)]
async fn inbreak_postpone_cleans_session() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(60)
        .mini_break_duration_s(20)
        .build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // Trigger break
    advance_time_and_yield(duration_s(60)).await;
    assert!(env.shared_state.read().in_any_session());

    // User postpones break
    cmd_tx.send(Command::PostponeBreak).await.unwrap();
    advance_time_and_yield(duration_ms(500)).await;

    // Session must be cleaned
    assert!(
        !env.shared_state.read().in_any_session(),
        "Session must be cleaned when postponing break"
    );

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **ST4: Session Recovery After Pause/Resume**
#[tokio::test(start_paused = true)]
async fn session_recovery_after_pause_resume() {
    let config = TestConfigBuilder::new()
        .mini_break_interval_s(30)
        .mini_break_duration_s(10)
        .build();

    let env = create_manager_test_env(config);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    spawn_test_manager(&env, cmd_rx).await;
    advance_time_and_yield(duration_ms(200)).await;

    // First break
    advance_time_and_yield(duration_s(30)).await;
    assert!(env.shared_state.read().in_any_session());

    // Pause during break
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(500)).await;
    assert!(!env.shared_state.read().in_any_session());

    // Resume
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(500)).await;

    // Wait for next break
    advance_time_and_yield(duration_s(35)).await;

    // Session should work normally
    assert!(
        env.shared_state.read().in_any_session(),
        "Session management should work after pause/resume cycle"
    );

    // Cleanup
    drop(cmd_tx);
    drop(env.shutdown_tx);
}

/// **Test: Config Update While Paused**
///
/// Verifies that updating config while paused doesn't resume scheduler.
/// This prevents state inconsistency where scheduler resumes but pause
/// reasons still exist in `SharedState`.
#[tokio::test(start_paused = true)]
async fn config_update_while_paused() {
    let initial = TestConfigBuilder::new().mini_break_interval_s(60).build();

    let (mut scheduler, emitter, shutdown_tx, _app) = create_test_break_scheduler(initial);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let task = tokio::spawn(async move {
        scheduler.run(cmd_rx).await;
    });

    advance_time_and_yield(duration_ms(200)).await;

    // Verify scheduler is running
    let status_before = get_latest_status(&emitter);
    assert!(!status_before.paused);
    assert!(status_before.next_event.is_some());

    // Pause the scheduler
    cmd_tx
        .send(Command::Pause(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // Verify scheduler is paused
    let status_paused = get_latest_status(&emitter);
    assert!(status_paused.paused);
    assert!(status_paused.next_event.is_none());

    // Update config while paused
    let new_config = TestConfigBuilder::new().mini_break_interval_s(120).build();
    cmd_tx
        .send(Command::UpdateConfig(new_config))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    // CRITICAL: Should stay paused after config update
    let status_after_update = get_latest_status(&emitter);
    assert!(
        status_after_update.paused,
        "Scheduler should stay paused after config update"
    );
    assert!(
        status_after_update.next_event.is_none(),
        "Should not have next event while paused"
    );

    // Resume should work normally with new config
    cmd_tx
        .send(Command::Resume(PauseReason::Manual))
        .await
        .unwrap();
    advance_time_and_yield(duration_ms(200)).await;

    let status_resumed = get_latest_status(&emitter);
    assert!(!status_resumed.paused);
    assert!(status_resumed.next_event.is_some());
    // Should use new interval (120s)
    assert_duration_near(
        status_resumed.next_event.unwrap().seconds_until.into(),
        120,
        5,
    );

    // Cleanup
    drop(cmd_tx);
    drop(shutdown_tx);
    task.await.unwrap();
}
