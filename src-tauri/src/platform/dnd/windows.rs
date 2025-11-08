//! Windows DND monitoring via WNF (Windows Notification Facility)
//!
//! This implementation uses the undocumented but stable WNF API to receive
//! real-time notifications when Focus Assist state changes. This provides
//! zero-polling event-driven monitoring.
//!
//! Focus Assist states:
//! - 0: Off
//! - 1: Priority Only
//! - 2: Alarms Only
//!
//! ref: <https://stackoverflow.com/questions/53407374/is-there-a-way-to-detect-changes-in-focus-assist-formerly-quiet-hours-in-windo>

use std::mem;
use std::sync::Arc;

use anyhow::{Context, Result};
use parking_lot::Mutex as ParkingMutex;
use tokio::sync::Mutex as TokioMutex;
use tokio::sync::mpsc;
use windows::Win32::Foundation::NTSTATUS;

use super::DndEvent;

const MIN_VALID_POINTER_ADDR: usize = 0x1000;
const MAX_BUFFER_SIZE: u32 = 1024;
const EXPECTED_STATE_SIZE: u32 = mem::size_of::<FocusAssistState>() as u32;

/// Wrapper to make `WnfUserSubscription` pointer Send
/// Safety: The WNF subscription is thread-safe and can be safely sent between threads
struct SendPtr(*mut WnfUserSubscription);

// SAFETY: WNF subscription handles are thread-safe
unsafe impl Send for SendPtr {}
unsafe impl Sync for SendPtr {}

impl SendPtr {
    fn new(ptr: *mut WnfUserSubscription) -> Self {
        Self(ptr)
    }

    fn as_ptr(&self) -> *mut WnfUserSubscription {
        self.0
    }
}

/// Windows DND monitor using WNF API
pub struct WindowsDndMonitor {
    is_monitoring: Arc<TokioMutex<bool>>,
    last_state: Arc<TokioMutex<bool>>,
    subscription: Arc<TokioMutex<Option<SendPtr>>>,
}

impl WindowsDndMonitor {
    /// Create a new Windows DND monitor
    pub fn new() -> Result<Self> {
        Ok(Self {
            is_monitoring: Arc::new(TokioMutex::new(false)),
            last_state: Arc::new(TokioMutex::new(false)),
            subscription: Arc::new(TokioMutex::new(None)),
        })
    }

    /// Start monitoring Focus Assist state changes
    pub async fn start(&mut self, sender: mpsc::Sender<DndEvent>) -> Result<()> {
        let mut is_monitoring = self.is_monitoring.lock().await;
        if *is_monitoring {
            tracing::debug!("Windows DND monitoring is already running");
            return Ok(());
        }

        tracing::info!("Starting Windows DND monitoring via WNF API");

        // Skip initial state query - callback will fire immediately with current state
        // This avoids potential issues with RtlQueryWnfStateData function signature
        let initial_state = false; // Assume disabled, will be updated by callback
        *self.last_state.lock().await = initial_state;

        // Subscribe to WNF notifications
        let callback_last_state = Arc::new(ParkingMutex::new(initial_state));
        match subscribe_to_focus_assist(sender, callback_last_state) {
            Ok(subscription) => {
                *self.subscription.lock().await = Some(subscription);
                *is_monitoring = true;
                tracing::info!("Windows DND monitoring started successfully");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to subscribe to Focus Assist notifications: {e}");
                *is_monitoring = false;
                Err(e).context("Windows DND monitor failed to start")
            }
        }
    }

    /// Stop monitoring Focus Assist state
    pub async fn stop(&mut self) -> Result<()> {
        let mut is_monitoring = self.is_monitoring.lock().await;
        if !*is_monitoring {
            return Ok(());
        }

        tracing::info!("Stopping Windows DND monitoring");

        // Unsubscribe from WNF notifications
        if let Some(subscription) = self.subscription.lock().await.take() {
            unsubscribe_from_focus_assist(subscription.as_ptr());
        }

        *is_monitoring = false;
        tracing::info!("Windows DND monitoring stopped");
        Ok(())
    }

    /// Get current Focus Assist state
    #[allow(clippy::unused_async)] // for consistency with other platforms
    pub async fn is_enabled(&self) -> Result<bool> {
        Ok(query_focus_assist_state())
    }
}

impl Drop for WindowsDndMonitor {
    fn drop(&mut self) {
        // **SAFETY**: Drop must never panic as it's called during unwinding
        // Wrap everything in catch_unwind to be absolutely safe
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Use blocking_lock in Drop since we're in a sync context
            // This is safe because Drop is called when the monitor is being destroyed
            if let Ok(mut subscription_guard) = self.subscription.try_lock() {
                if let Some(subscription) = subscription_guard.take() {
                    unsubscribe_from_focus_assist(subscription.as_ptr());
                }
            } else {
                // If we can't get the lock, just log and continue
                // The subscription will leak but the app won't crash
                tracing::warn!(
                    "Could not acquire subscription lock during drop. Subscription may leak."
                );
            }
        }));
    }
}

// ============================================================================
// WNF FFI Definitions
// ============================================================================

/// WNF State Name for Focus Assist profile changes
const WNF_SHEL_QUIETHOURS_ACTIVE_PROFILE_CHANGED: WnfStateName = WnfStateName {
    data: [0xA3BF_1C75, 0x0D83_063E],
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct WnfStateName {
    data: [u32; 2],
}

#[repr(C)]
struct WnfUserSubscription {
    _internal: [u8; 256], // Opaque handle
}

type WnfChangeStamp = u32;

/// Focus Assist state structure
#[repr(C)]
struct FocusAssistState {
    value: i32,
}

/// WNF callback function pointer
type WnfUserCallback = unsafe extern "system" fn(
    state_name: WnfStateName,
    change_stamp: WnfChangeStamp,
    type_id: *const (),
    callback_context: *mut CallbackContext,
    buffer: *const u8,
    length: u32,
) -> NTSTATUS;

/// Context passed to WNF callback
struct CallbackContext {
    sender: mpsc::Sender<DndEvent>,
    last_state: Arc<ParkingMutex<bool>>,
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn RtlQueryWnfStateData(
        change_stamp: *mut WnfChangeStamp,
        state_name: WnfStateName,
        callback: Option<WnfUserCallback>,
        callback_context: *mut (),
        buffer: *mut (),
    ) -> NTSTATUS;

    fn RtlSubscribeWnfStateChangeNotification(
        subscription: *mut *mut WnfUserSubscription,
        state_name: WnfStateName,
        change_stamp: WnfChangeStamp,
        callback: WnfUserCallback,
        callback_context: *mut (),
        type_id: *const (),
        serialization_group: u32,
        flags: u32,
    ) -> NTSTATUS;

    fn RtlUnsubscribeWnfStateChangeNotification(subscription: *mut WnfUserSubscription)
    -> NTSTATUS;
}

// ============================================================================
// WNF Helper Functions
// ============================================================================

/// Query the current Focus Assist state
///
/// # Safety
/// This function uses undocumented Windows WNF API. All errors are caught and logged
/// without causing panics. Returns `false` (DND disabled) on any error to fail safely.
fn query_focus_assist_state() -> bool {
    // Wrap the entire unsafe block in a catch_unwind to prevent panics from propagating
    std::panic::catch_unwind(|| unsafe {
        let mut change_stamp: WnfChangeStamp = 0;
        let mut state = FocusAssistState { value: 0 };

        let status = RtlQueryWnfStateData(
            &raw mut change_stamp,
            WNF_SHEL_QUIETHOURS_ACTIVE_PROFILE_CHANGED,
            None,
            std::ptr::null_mut(),
            (&raw mut state).cast::<()>(),
        );

        if status.is_err() {
            tracing::warn!(
                "RtlQueryWnfStateData failed with status: {status:?}. This is non-fatal."
            );
            return false;
        }

        // Validate state value is within expected range (0-2)
        if state.value < 0 || state.value > 2 {
            tracing::warn!(
                "Unexpected Focus Assist state value: {}. Treating as disabled.",
                state.value
            );
            return false;
        }

        // 0 = Off, 1 = Priority Only, 2 = Alarms Only
        state.value != 0
    })
    .unwrap_or_else(|panic_err| {
        tracing::error!(
            "Panic caught in query_focus_assist_state: {panic_err:?}. This is a safety fallback.",
        );
        false // Return false (disabled) as safe default
    })
}

/// Subscribe to Focus Assist state change notifications
///
/// # Safety
/// This function uses undocumented Windows WNF API. All errors are caught and logged.
/// Memory cleanup is guaranteed even on failure paths to prevent leaks.
fn subscribe_to_focus_assist(
    sender: mpsc::Sender<DndEvent>,
    last_state: Arc<ParkingMutex<bool>>,
) -> Result<SendPtr> {
    // Wrap the entire unsafe block in a catch_unwind to prevent panics from propagating
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        let context = Box::new(CallbackContext { sender, last_state });
        let context_ptr = Box::into_raw(context).cast::<()>();

        let mut subscription: *mut WnfUserSubscription = std::ptr::null_mut();

        let status = RtlSubscribeWnfStateChangeNotification(
            &raw mut subscription,
            WNF_SHEL_QUIETHOURS_ACTIVE_PROFILE_CHANGED,
            0, // Change stamp
            focus_assist_callback,
            context_ptr,
            std::ptr::null(),
            0, // No serialization group
            0, // No flags
        );

        if status.is_err() {
            // Clean up context if subscription failed
            let _ = Box::from_raw(context_ptr.cast::<CallbackContext>());
            tracing::error!(
                "RtlSubscribeWnfStateChangeNotification failed: {status:?}. DND monitoring will be unavailable."
            );
            return Err(anyhow::anyhow!(
                "Failed to subscribe to WNF notifications: {status:?}"
            ));
        }

        if !is_valid_ptr(subscription) {
            let _ = Box::from_raw(context_ptr.cast::<CallbackContext>());
            tracing::error!("WNF subscription handle appears invalid: {subscription:p}");
            return Err(anyhow::anyhow!("Invalid subscription handle"));
        }

        // Return the subscription pointer wrapped in SendPtr
        Ok(SendPtr::new(subscription))
    }));

    match result {
        Ok(Ok(ptr)) => Ok(ptr),
        Ok(Err(e)) => Err(e),
        Err(panic_err) => {
            tracing::error!(
                "Panic caught in subscribe_to_focus_assist: {panic_err:?}. This prevents app crash.",
            );
            Err(anyhow::anyhow!(
                "Panic occurred during WNF subscription: {panic_err:?}"
            ))
        }
    }
}

/// Unsubscribe from Focus Assist notifications
///
/// # Safety
/// This function uses undocumented Windows WNF API. All errors are caught and logged
/// without propagating. Panics are caught to prevent app crashes during cleanup.
#[allow(clippy::unnecessary_wraps)]
fn unsubscribe_from_focus_assist(subscription: *mut WnfUserSubscription) {
    if !is_valid_ptr(subscription) {
        tracing::warn!("Invalid subscription pointer during unsubscribe: {subscription:p}");
        return; // Non-fatal, just skip unsubscribe
    }

    // Wrap in catch_unwind to prevent panics during cleanup
    let result = std::panic::catch_unwind(|| unsafe {
        let status = RtlUnsubscribeWnfStateChangeNotification(subscription);
        if status.is_err() {
            tracing::error!(
                "Failed to unsubscribe from WNF notifications: {status:?}. This is non-fatal."
            );
        }
    });

    if let Err(panic_err) = result {
        tracing::error!(
            "Panic caught during WNF unsubscribe: {panic_err:?}. Memory may leak but app continues."
        );
    }
}

/// WNF callback invoked when Focus Assist state changes
///
/// # Safety
/// This is a system callback that must never panic. All operations are wrapped in
/// panic handlers and extensive validation to ensure callback safety.
///
/// **CRITICAL**: This function is called by Windows. Any panic here could crash the app
/// or leave Windows in a bad state. All code paths must be panic-safe.
unsafe extern "system" fn focus_assist_callback(
    _state_name: WnfStateName,
    _change_stamp: WnfChangeStamp,
    _type_id: *const (),
    callback_context: *mut CallbackContext,
    buffer: *const u8,
    length: u32,
) -> NTSTATUS {
    // **CRITICAL SAFETY BOUNDARY**: Wrap everything in catch_unwind
    // This is a system callback - we MUST NOT panic
    // We need to move pointer values into owned variables before catch_unwind
    let ctx_ptr = callback_context;
    let buf_ptr = buffer;
    let buf_len = length;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        // Validate all pointers before dereferencing
        if !is_valid_ptr(ctx_ptr) {
            tracing::error!("WNF callback: invalid callback_context pointer: {ctx_ptr:p}");
            return NTSTATUS(0);
        }
        if !is_valid_ptr(buf_ptr) {
            tracing::error!("WNF callback: invalid buffer pointer: {buf_ptr:p}");
            return NTSTATUS(0);
        }

        // Validate buffer size
        let expected_size = EXPECTED_STATE_SIZE;
        if buf_len < expected_size {
            tracing::error!(
                "WNF callback: buffer too small. Expected at least {expected_size} bytes, got {buf_len}"
            );
            return NTSTATUS(0);
        }

        // Validate buffer size is not unreasonably large (detect corruption)
        if buf_len > MAX_BUFFER_SIZE {
            tracing::error!(
                "WNF callback: buffer suspiciously large: {buf_len} bytes. Possible corruption."
            );
            return NTSTATUS(0);
        }

        // Parse Focus Assist state with additional safety
        let mut state_aligned = FocusAssistState { value: 0 };
        std::ptr::copy_nonoverlapping(
            buf_ptr,
            (&raw mut state_aligned).cast::<u8>(),
            EXPECTED_STATE_SIZE as usize,
        );
        let state = state_aligned;

        // Validate state value is within expected range
        if state.value < 0 || state.value > 2 {
            tracing::warn!(
                "WNF callback: unexpected Focus Assist state value: {}. Processing anyway.",
                state.value
            );
        }

        let is_active = state.value != 0; // 0 = Off, 1 = Priority Only, 2 = Alarms Only

        // Get context with validation
        let context = &*ctx_ptr;

        // Try to lock the state mutex - if this fails, skip this update
        let Some(mut last_state) = context.last_state.try_lock() else {
            tracing::debug!(
                "WNF callback: could not acquire lock on last_state. Skipping this update."
            );
            return NTSTATUS(0);
        };

        // Only emit event if state actually changed
        if *last_state != is_active {
            *last_state = is_active;

            let event = if is_active {
                DndEvent::Started
            } else {
                DndEvent::Finished
            };

            tracing::info!("Focus Assist state changed: {}", event.description());

            // Send event asynchronously using Tauri runtime (not tokio::spawn, as we're not in a tokio context)
            // This callback is executed on a Windows thread pool thread, not a tokio thread
            let sender = context.sender.clone();
            tauri::async_runtime::spawn(async move {
                sender.send(event).await.unwrap_or_else(|e| {
                    tracing::error!("Failed to send DND event: {e}");
                });
            });
        }

        NTSTATUS(0) // Success
    }));

    match result {
        Ok(status) => status,
        Err(panic_err) => {
            // **CRITICAL**: A panic occurred in the callback
            // Log it and return success to Windows to prevent further issues
            tracing::error!(
                "CRITICAL: Panic caught in WNF callback: {panic_err:?}. Returning success to Windows to prevent system instability."
            );
            NTSTATUS(0)
        }
    }
}

fn is_valid_ptr<T>(ptr: *const T) -> bool {
    !ptr.is_null() && (ptr as usize) >= MIN_VALID_POINTER_ADDR
}
