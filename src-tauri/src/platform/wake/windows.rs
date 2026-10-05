use std::{ffi::c_void, ptr};

use anyhow::{Context, Result};
use tauri::AppHandle;
use windows::Win32::{
    Foundation::HANDLE,
    System::Power::{
        DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS, HPOWERNOTIFY, PowerRegisterSuspendResumeNotification,
        PowerUnregisterSuspendResumeNotification,
    },
    UI::WindowsAndMessaging::{DEVICE_NOTIFY_CALLBACK, PBT_APMRESUMEAUTOMATIC},
};

pub(crate) struct WakeMonitor {
    registration: HPOWERNOTIFY,
    // A stable address retained until the native callback is unregistered.
    context: Option<Box<AppHandle>>,
}

impl WakeMonitor {
    pub(crate) fn new(app: &AppHandle) -> Result<Self> {
        let mut context = Box::new(app.clone());
        let mut parameters = DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
            Callback: Some(on_power_event),
            Context: ptr::from_mut(context.as_mut()).cast(),
        };
        let mut registration = ptr::null_mut();
        // SAFETY: The descriptor is valid for registration. Its callback context
        // is boxed and kept alive until unregistering in Drop.
        unsafe {
            PowerRegisterSuspendResumeNotification(
                DEVICE_NOTIFY_CALLBACK,
                HANDLE(ptr::from_mut(&mut parameters).cast()),
                &raw mut registration,
            )
            .ok()
        }
        .context("Failed to subscribe to Windows wake notifications")?;
        Ok(Self {
            registration: HPOWERNOTIFY(registration as isize),
            context: Some(context),
        })
    }
}

impl Drop for WakeMonitor {
    fn drop(&mut self) {
        // SAFETY: This handle came from successful registration and is removed
        // before its callback context is freed.
        let result = unsafe { PowerUnregisterSuspendResumeNotification(self.registration) }.ok();
        if let Err(error) = result {
            tracing::warn!("Could not unregister Windows wake notifications: {error}");
            // A failed unregister must not leave a callback pointing to freed memory.
            if let Some(context) = self.context.take() {
                Box::leak(context);
            }
        }
    }
}

unsafe extern "system" fn on_power_event(
    context: *const c_void,
    event: u32,
    _setting: *const c_void,
) -> u32 {
    // Windows also emits RESUMESUSPEND on user activity. Handling only AUTOMATIC
    // avoids resetting twice for the same wake and includes background wakes.
    if event == PBT_APMRESUMEAUTOMATIC {
        // SAFETY: Registration supplies this boxed AppHandle and owns its lifetime.
        let app = unsafe { &*context.cast::<AppHandle>() };
        super::notify_resume(app);
    }
    0
}
