//! System wake notifications only: no polling, sleep timing, or battery monitoring.
//!
//! The guard is created and dropped on Tauri's event-loop thread. In particular,
//! macOS notification objects never need to cross threads.

use tauri::{AppHandle, Manager};

use crate::{cmd::SchedulerCmd, scheduler::Command};

#[cfg(target_os = "linux")]
#[path = "wake/linux.rs"]
mod native;
#[cfg(target_os = "macos")]
#[path = "wake/macos.rs"]
mod native;
#[cfg(windows)]
#[path = "wake/windows.rs"]
mod native;

pub(crate) use native::WakeMonitor;

fn notify_resume(app: &AppHandle) {
    // Notifications can arrive before asynchronous application setup completes.
    if let Some(scheduler) = app.try_state::<SchedulerCmd>() {
        scheduler.try_send_command(&Command::SystemResumed);
    }
}
