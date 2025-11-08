use tauri::State;

use crate::core::payload::{PromptPayload, PromptPayloadStore};

/// Store a prompt payload in the backend
#[tauri::command]
pub async fn store_prompt_payload(
    state: State<'_, PromptPayloadStore>,
    payload: PromptPayload,
    payload_id: String,
) -> Result<(), String> {
    state.store(payload_id, payload).await;
    Ok(())
}

/// Get a prompt payload from the backend
#[tauri::command]
pub async fn get_prompt_payload(
    payload_id: String,
    state: State<'_, PromptPayloadStore>,
) -> Result<PromptPayload, String> {
    state
        .get(&payload_id)
        .await
        .ok_or_else(|| format!("Prompt payload not found: {payload_id}"))
}

/// Remove a prompt payload from the backend (cleanup)
#[tauri::command]
pub async fn remove_prompt_payload(
    payload_id: String,
    state: State<'_, PromptPayloadStore>,
) -> Result<(), String> {
    state.remove(&payload_id).await;
    Ok(())
}
