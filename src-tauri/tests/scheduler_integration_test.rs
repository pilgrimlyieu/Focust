use focust_lib::config::AppConfig;
use focust_lib::scheduler::core::Scheduler;
use focust_lib::scheduler::event::{
    AttentionEventSource, BreakEventSource, EventSource, SchedulingContext,
};
use focust_lib::scheduler::models::EventKind;

use chrono::{Duration, TimeZone, Utc};
use tauri::Listener;
use tokio::sync::{mpsc, watch};

const MINI_BREAK_INTERVAL_S: u64 = 600;
const AFTER_MINI_BREAK_TIMES: u8 = 2;
const NOTIFICATION_BEFORE_S: u64 = 10;

#[allow(dead_code)]
fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
    builder
        .setup(|app| {
            app.listen_any("new_event", move |_event| println!("event handler called"));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
}

#[allow(dead_code)]
fn setup_test_scheduler() -> Scheduler {
    let (_cmd_tx, cmd_rx) = mpsc::channel(1);
    let (_shutdown_tx, shutdown_rx) = watch::channel(());
    let app_handle = create_app(tauri::Builder::default()).handle().clone();

    let sources: Vec<Box<dyn EventSource>> =
        vec![Box::new(BreakEventSource), Box::new(AttentionEventSource)];

    Scheduler::new(app_handle, cmd_rx, shutdown_rx, sources)
}

#[test]
fn test_break_event_is_scheduled_correctly() {
    // 1. Settings
    let source = BreakEventSource;
    let mut config = AppConfig::default();
    config.schedules[0].enabled = true;
    config.schedules[0].mini_breaks.interval_s = MINI_BREAK_INTERVAL_S;
    config.schedules[0].notification_before_s = NOTIFICATION_BEFORE_S;

    let fake_now = Utc.with_ymd_and_hms(2025, 9, 1, 12, 0, 0).unwrap();

    let context = SchedulingContext {
        now_utc: fake_now,
        now_local: fake_now.with_timezone(&chrono::Local),
        config: &config,
        mini_break_counter: 0,
        last_break_time: None, // First time break
    };

    // 2. Execute
    let events = source.upcoming_events(&context);

    // 3. Assert
    assert_eq!(events.len(), 2);

    let expected_break_time = fake_now + Duration::seconds(MINI_BREAK_INTERVAL_S as i64);
    let break_event = events
        .iter()
        .find(|e| matches!(e.kind, EventKind::MiniBreak(_)))
        .unwrap();
    assert_eq!(break_event.time, expected_break_time);

    let expected_notification_time =
        expected_break_time - Duration::seconds(NOTIFICATION_BEFORE_S as i64);
    let notification_event = events
        .iter()
        .find(|e| matches!(e.kind, EventKind::Notification(_)))
        .unwrap();
    assert_eq!(notification_event.time, expected_notification_time);
}

#[test]
fn test_long_break_overrides_mini_break() {
    // 1. Settings
    let source = BreakEventSource;
    let mut config = AppConfig::default();
    config.schedules[0].enabled = true;
    config.schedules[0].long_breaks.after_mini_breaks = AFTER_MINI_BREAK_TIMES;

    let fake_now = Utc.with_ymd_and_hms(2025, 9, 1, 12, 0, 0).unwrap();
    let context = focust_lib::scheduler::event::SchedulingContext {
        now_utc: fake_now,
        now_local: fake_now.with_timezone(&chrono::Local),
        config: &config,
        mini_break_counter: 2, // Reached the threshold for a long break
        last_break_time: Some(fake_now - Duration::minutes(MINI_BREAK_INTERVAL_S as i64 / 60)),
    };

    // 2. Execute
    let events = source.upcoming_events(&context);

    // 3. Assert
    let break_event = events
        .iter()
        .find(|e| matches!(e.kind, EventKind::LongBreak(_)));
    assert!(break_event.is_some());

    let mini_break_event = events
        .iter()
        .find(|e| matches!(e.kind, EventKind::MiniBreak(_)));
    assert!(mini_break_event.is_none());
}
