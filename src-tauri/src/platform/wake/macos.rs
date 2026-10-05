use std::ptr::NonNull;

use anyhow::Result;
use block2::RcBlock;
use objc2::{
    rc::Retained,
    runtime::{NSObjectProtocol, ProtocolObject},
};
use objc2_app_kit::{NSWorkspace, NSWorkspaceDidWakeNotification};
use objc2_foundation::{NSNotification, NSNotificationCenter};
use tauri::AppHandle;

pub(crate) struct WakeMonitor {
    center: Retained<NSNotificationCenter>,
    observer: Retained<ProtocolObject<dyn NSObjectProtocol>>,
}

impl WakeMonitor {
    #[expect(
        clippy::unnecessary_wraps,
        reason = "Common constructor across desktop platforms"
    )]
    pub(crate) fn new(app: &AppHandle) -> Result<Self> {
        let app = app.clone();
        let center = NSWorkspace::sharedWorkspace().notificationCenter();
        let callback = RcBlock::new(move |_: NonNull<NSNotification>| {
            super::notify_resume(&app);
        });
        // SAFETY: The captured AppHandle is Send + Sync, no object filter or
        // operation queue is supplied, and the center retains the copied block.
        let observer = unsafe {
            center.addObserverForName_object_queue_usingBlock(
                Some(NSWorkspaceDidWakeNotification),
                None,
                None,
                &callback,
            )
        };
        Ok(Self { center, observer })
    }
}

impl Drop for WakeMonitor {
    fn drop(&mut self) {
        // SAFETY: This token was returned by this center's observer registration.
        unsafe { self.center.removeObserver(self.observer.as_ref()) };
    }
}
