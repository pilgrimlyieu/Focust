//! Event emission abstraction for scheduler testing
//!
//! This module provides a trait-based abstraction for event emission,
//! allowing schedulers to be tested without requiring a real Tauri `AppHandle`.

#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use parking_lot::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Trait for emitting events from schedulers
///
/// This abstraction allows for dependency injection of event emission,
/// making schedulers testable without requiring a full Tauri runtime.
///
/// The `emit` method signature matches Tauri's `Emitter::emit` for consistency.
pub trait EventEmitter: Send + Sync {
    /// Emit an event with the given name and payload
    ///
    /// # Arguments
    /// * `event` - The event name (e.g., "scheduler-event")
    /// * `payload` - The event payload as a serializable value
    ///
    /// # Returns
    /// Ok(()) if emission succeeded, Err with message if failed
    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<(), String>;
}

/// Production implementation using Tauri's `AppHandle`
///
/// This is the real implementation used in the application.
/// It wraps `AppHandle.emit()` with our `EventEmitter` interface.
pub struct TauriEventEmitter {
    app_handle: AppHandle,
}

impl TauriEventEmitter {
    /// Create a new `TauriEventEmitter`
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl EventEmitter for TauriEventEmitter {
    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<(), String> {
        self.app_handle
            .emit(event, payload)
            .map_err(|e| e.to_string())
    }
}

/// Test implementation that records emitted events
///
/// This implementation stores all emitted events in a Vec,
/// allowing tests to verify that correct events were emitted.
#[cfg(test)]
#[derive(Clone)]
pub struct TestEventEmitter {
    events: Arc<Mutex<Vec<(String, serde_json::Value)>>>,
}

#[cfg(test)]
impl TestEventEmitter {
    /// Create a new `TestEventEmitter`
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get all emitted events
    pub fn get_events(&self) -> Vec<(String, serde_json::Value)> {
        self.events.lock().clone()
    }

    /// Get events by name
    pub fn get_events_by_name(&self, event_name: &str) -> Vec<serde_json::Value> {
        self.events
            .lock()
            .iter()
            .filter(|(name, _)| name == event_name)
            .map(|(_, payload)| payload.clone())
            .collect()
    }

    /// Clear all recorded events
    pub fn clear(&self) {
        self.events.lock().clear();
    }

    /// Get the number of emitted events
    pub fn event_count(&self) -> usize {
        self.events.lock().len()
    }

    /// Check if a specific event was emitted
    pub fn has_event(&self, event_name: &str) -> bool {
        self.events
            .lock()
            .iter()
            .any(|(name, _)| name == event_name)
    }
}

#[cfg(test)]
impl EventEmitter for TestEventEmitter {
    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<(), String> {
        let json_value = serde_json::to_value(payload)
            .map_err(|e| format!("Failed to serialize payload: {e}"))?;
        self.events.lock().push((event.to_string(), json_value));
        Ok(())
    }
}

#[cfg(test)]
impl Default for TestEventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_event_emitter_records_events() {
        let emitter = TestEventEmitter::new();

        emitter.emit("test-event", json!({"key": "value"})).unwrap();
        emitter.emit("another-event", json!({"num": 42})).unwrap();

        let events = emitter.get_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].0, "test-event");
        assert_eq!(events[1].0, "another-event");
    }

    #[test]
    fn test_get_events_by_name() {
        let emitter = TestEventEmitter::new();

        emitter.emit("event-a", json!({"id": 1})).unwrap();
        emitter.emit("event-b", json!({"id": 2})).unwrap();
        emitter.emit("event-a", json!({"id": 3})).unwrap();

        let events = emitter.get_events_by_name("event-a");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["id"], 1);
        assert_eq!(events[1]["id"], 3);
    }

    #[test]
    fn test_clear_events() {
        let emitter = TestEventEmitter::new();

        emitter.emit("test", json!({})).unwrap();
        assert_eq!(emitter.event_count(), 1);

        emitter.clear();
        assert_eq!(emitter.event_count(), 0);
    }

    #[test]
    fn test_has_event() {
        let emitter = TestEventEmitter::new();

        emitter.emit("exists", json!({})).unwrap();

        assert!(emitter.has_event("exists"));
        assert!(!emitter.has_event("does-not-exist"));
    }
}
