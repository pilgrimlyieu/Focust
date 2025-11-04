use crate::core::audio;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

/// Tauri command to play an audio file
#[tauri::command]
pub async fn play_audio(path: String, volume: f32) -> Result<(), String> {
    audio::play_audio(&path, volume)
        .await
        .map_err(|e| format!("Failed to play audio: {e}"))
}

/// Tauri command to play a builtin audio resource
/// Automatically resolves the resource path from the bundled assets
#[tauri::command]
pub async fn play_builtin_audio(
    app: AppHandle,
    resource_name: String,
    volume: f32,
) -> Result<(), String> {
    // Resolve the resource path
    let resource_path = resolve_builtin_audio_path(&app, &resource_name)?;

    // Play the audio
    audio::play_audio(&resource_path, volume)
        .await
        .map_err(|e| format!("Failed to play builtin audio: {e}"))
}

/// Tauri command to stop audio playback
#[tauri::command]
pub async fn stop_audio() -> Result<(), String> {
    audio::stop_audio()
        .await
        .map_err(|e| format!("Failed to stop audio: {e}"))
}

/// Helper function to resolve the path of a builtin audio resource
fn resolve_builtin_audio_path(app: &AppHandle, resource_name: &str) -> Result<String, String> {
    let resource_relative_path = format!("assets/sounds/{resource_name}.mp3");

    tracing::debug!("Attempting to resolve builtin audio resource: {resource_relative_path}",);

    let resolved_path_buf = app
        .path()
        .resolve(&resource_relative_path, BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve resource path for '{resource_name}': {e}"))?;

    tracing::debug!(
        "Resolved builtin audio path: {}",
        resolved_path_buf.display()
    );

    if !resolved_path_buf.exists() {
        return Err(format!(
            "Builtin audio resource '{resource_name}' not found at resolved path: {}",
            resolved_path_buf.display()
        ));
    }

    resolved_path_buf
        .to_str()
        .ok_or_else(|| "Invalid path encoding".to_string())
        .map(std::string::ToString::to_string)
}
