use std::sync::Arc;
use std::{collections::HashMap, ops::Deref};

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::RwLock;
use ts_rs::TS;

use crate::core::audio::AudioSettings;
use crate::core::theme::ThemeSettings;

/// Break kind type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub enum BreakKind {
    Mini,
    Long,
    Attention,
}

/// Resolved background for break window
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub enum ResolvedBackground {
    Solid { value: String },
    Image { value: String },
}

/// Break payload stored in backend
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct BreakPayload {
    pub id: u32,
    pub kind: BreakKind,
    pub title: String,
    pub message_key: String,
    pub message: Option<String>,
    pub duration: u32,
    pub strict_mode: bool,
    pub theme: ThemeSettings,
    pub background: ResolvedBackground,
    pub suggestion: Option<String>,
    pub audio: Option<AudioSettings>,
    pub all_screens: bool,
    pub schedule_name: Option<String>,
    pub postpone_shortcut: String,
}

/// Shared state for storing active break payloads
pub struct BreakPayloadStore(Arc<RwLock<HashMap<String, BreakPayload>>>);

impl Default for BreakPayloadStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BreakPayloadStore {
    type Target = Arc<RwLock<HashMap<String, BreakPayload>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BreakPayloadStore {
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    /// Store a break payload with a unique identifier
    pub async fn store(&self, payload_id: String, payload: BreakPayload) {
        let mut payloads = self.write().await;
        tracing::debug!("Storing break payload with id: {payload_id}");
        payloads.insert(payload_id, payload);
    }

    /// Retrieve a break payload by identifier
    pub async fn get(&self, payload_id: &str) -> Option<BreakPayload> {
        let payloads = self.read().await;
        tracing::debug!("Retrieving break payload with id: {payload_id}");
        payloads.get(payload_id).cloned()
    }

    /// Remove a break payload after it's been consumed
    pub async fn remove(&self, payload_id: &str) {
        let mut payloads = self.write().await;
        tracing::debug!("Removing break payload with id: {payload_id}");
        payloads.remove(payload_id);
    }

    /// Clear all stored payloads
    pub async fn clear(&self) {
        let mut payloads = self.write().await;
        payloads.clear();
        tracing::debug!("Cleared all break payloads");
    }
}

/// Store a break payload in the backend
#[tauri::command]
pub async fn store_break_payload(
    payload_id: String,
    payload: BreakPayload,
    state: State<'_, BreakPayloadStore>,
) -> Result<(), String> {
    state.store(payload_id, payload).await;
    Ok(())
}

/// Get a break payload from the backend
#[tauri::command]
pub async fn get_break_payload(
    payload_id: String,
    state: State<'_, BreakPayloadStore>,
) -> Result<BreakPayload, String> {
    state
        .get(&payload_id)
        .await
        .ok_or_else(|| format!("Break payload not found: {payload_id}"))
}

/// Remove a break payload from the backend (cleanup)
#[tauri::command]
pub async fn remove_break_payload(
    payload_id: String,
    state: State<'_, BreakPayloadStore>,
) -> Result<(), String> {
    state.remove(&payload_id).await;
    Ok(())
}
