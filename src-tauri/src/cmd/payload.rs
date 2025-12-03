//! Tauri commands for managing break prompt payloads.
//!
//! This module provides commands to store, retrieve, and remove break prompt
//! payloads used when displaying break windows.

use tauri::State;

use crate::core::payload::{PromptPayload, PromptPayloadStore};

/// Stores a prompt payload in the backend.
///
/// The payload is associated with a unique ID for later retrieval when
/// displaying the break prompt window.
///
/// # Errors
///
/// This function does not return errors in normal operation.
#[tauri::command]
pub async fn store_prompt_payload(
    state: State<'_, PromptPayloadStore>,
    payload: PromptPayload,
    payload_id: String,
) -> Result<(), String> {
    state.store(payload_id, payload).await;
    Ok(())
}

/// Retrieves a prompt payload from the backend by ID.
///
/// # Errors
///
/// Returns an error if the payload with the specified ID is not found.
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

/// Removes a prompt payload from the backend (cleanup).
///
/// Called after a break window is closed to free up memory.
///
/// # Errors
///
/// This function does not return errors in normal operation.
#[tauri::command]
pub async fn remove_prompt_payload(
    payload_id: String,
    state: State<'_, PromptPayloadStore>,
) -> Result<(), String> {
    state.remove(&payload_id).await;
    Ok(())
}
