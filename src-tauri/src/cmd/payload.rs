use tauri::State;

use crate::core::payload::{BreakPayload, BreakPayloadStore};

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
