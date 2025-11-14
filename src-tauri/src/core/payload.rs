use std::{collections::HashMap, ops::Deref, sync::Arc};

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};
use tokio::sync::RwLock;
use ts_rs::TS;

use crate::core::audio::AudioSettings;
use crate::core::theme::{ResolvedBackground, ThemeSettings};

/// Break kind type
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString, EnumIter, TS,
)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
#[strum(serialize_all = "PascalCase")]
pub enum EventKind {
    #[strum(serialize = "MiniBreak")]
    Mini,
    #[strum(serialize = "LongBreak")]
    Long,
    Attention,
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

/// Prompt payload stored in backend
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct PromptPayload {
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
    #[ts(type = "import('@/i18n').LocaleKey")]
    pub language: String,
    /// Number of times this break has been postponed
    pub postpone_count: u8,
    /// Maximum number of times this break can be postponed
    pub max_postpone_count: u8,
}

/// Shared state for storing active prompt payloads
pub struct PromptPayloadStore(Arc<RwLock<HashMap<String, PromptPayload>>>);

impl Default for PromptPayloadStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for PromptPayloadStore {
    type Target = Arc<RwLock<HashMap<String, PromptPayload>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PromptPayloadStore {
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    /// Store a prompt payload with a unique identifier
    pub async fn store(&self, payload_id: String, payload: PromptPayload) {
        let mut payloads = self.write().await;
        tracing::debug!("Storing prompt payload with id: {payload_id}");
        payloads.insert(payload_id, payload);
    }

    /// Retrieve a prompt payload by identifier
    pub async fn get(&self, payload_id: &str) -> Option<PromptPayload> {
        let payloads = self.read().await;
        tracing::debug!("Retrieving prompt payload with id: {payload_id}");
        payloads.get(payload_id).cloned()
    }

    /// Remove a prompt payload after it's been consumed
    pub async fn remove(&self, payload_id: &str) {
        let mut payloads = self.write().await;
        tracing::debug!("Removing prompt payload with id: {payload_id}");
        payloads.remove(payload_id);
    }

    /// Clear all stored payloads
    pub async fn clear(&self) {
        let mut payloads = self.write().await;
        payloads.clear();
        tracing::debug!("Cleared all prompt payloads");
    }
}

/// Store a prompt payload (non-command version for internal use)
pub async fn store_payload_internal(
    store: &PromptPayloadStore,
    payload: PromptPayload,
    payload_id: String,
) -> Result<(), String> {
    store.store(payload_id, payload).await;
    Ok(())
}
