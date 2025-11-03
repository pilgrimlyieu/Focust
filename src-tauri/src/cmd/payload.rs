use std::{collections::HashMap, ops::Deref};
use std::{fmt::Display, sync::Arc};

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::RwLock;
use ts_rs::TS;

use crate::core::theme::ThemeSettings;
use crate::core::{audio::AudioSettings, theme::HexColor};

/// Break kind type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub enum EventKind {
    Mini,
    Long,
    Attention,
}

impl Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind_str = match self {
            EventKind::Mini => "MiniBreak",
            EventKind::Long => "LongBreak",
            EventKind::Attention => "Attention",
        };
        write!(f, "{kind_str}")
    }
}

impl EventKind {
    #[must_use]
    pub fn is_break(&self) -> bool {
        matches!(self, EventKind::Mini | EventKind::Long)
    }

    #[must_use]
    pub fn is_attention(&self) -> bool {
        matches!(self, EventKind::Attention)
    }

    #[must_use]
    pub fn is_mini(&self) -> bool {
        matches!(self, EventKind::Mini)
    }

    #[must_use]
    pub fn is_long(&self) -> bool {
        matches!(self, EventKind::Long)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
#[derive(Default)]
pub enum BackgroundKind {
    #[default]
    Solid,
    Image,
}

impl Display for BackgroundKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind_str = match self {
            BackgroundKind::Solid => "solid",
            BackgroundKind::Image => "image",
        };
        write!(f, "{kind_str}")
    }
}

/// Resolved background for break window
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct ResolvedBackground {
    pub kind: BackgroundKind,
    pub value: String,
}

impl Default for ResolvedBackground {
    fn default() -> Self {
        Self {
            kind: BackgroundKind::Solid,
            value: HexColor::default().to_string(),
        }
    }
}

impl ResolvedBackground {
    #[must_use]
    pub fn new_solid(color: String) -> Self {
        Self {
            kind: BackgroundKind::Solid,
            value: color,
        }
    }

    #[must_use]
    pub fn new_image(path: String) -> Self {
        Self {
            kind: BackgroundKind::Image,
            value: path,
        }
    }

    #[must_use]
    pub fn is_solid(&self) -> bool {
        self.kind == BackgroundKind::Solid
    }

    #[must_use]
    pub fn is_image(&self) -> bool {
        self.kind == BackgroundKind::Image
    }
}

/// Break payload stored in backend
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct BreakPayload {
    pub id: u32,
    pub kind: EventKind,
    pub title: String,
    pub message_key: String,
    pub message: Option<String>,
    pub schedule_name: Option<String>,
    pub duration: i32,
    pub strict_mode: bool,
    pub theme: ThemeSettings,
    pub background: ResolvedBackground,
    pub suggestion: Option<String>,
    pub audio: Option<AudioSettings>,
    pub postpone_shortcut: String,
    pub all_screens: bool,
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
    state: State<'_, BreakPayloadStore>,
    payload: BreakPayload,
    payload_id: String,
) -> Result<(), String> {
    state.store(payload_id, payload).await;
    Ok(())
}

/// Store a break payload (non-command version for internal use)
pub async fn store_payload_internal(
    store: &BreakPayloadStore,
    payload: BreakPayload,
    payload_id: String,
) -> Result<(), String> {
    store.store(payload_id, payload).await;
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
