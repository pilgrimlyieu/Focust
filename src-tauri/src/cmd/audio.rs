/// Tauri commands for audio playback control
///
/// **Note**: Audio is temporarily disabled on macOS due to cpal Send trait limitations.
/// See `src/core/audio.rs` for detailed explanation and restoration plan.
/// Expected to be resolved in cpal 0.17.0+
use crate::core::audio;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

#[cfg(not(target_os = "macos"))]
use crate::core::audio::AudioPlayerState;
#[cfg(not(target_os = "macos"))]
use tauri::State;

/// Tauri command to play an audio file (non-macOS)
#[cfg(not(target_os = "macos"))]
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn play_audio(
    player: State<'_, AudioPlayerState>,
    path: String,
    volume: f32,
) -> Result<(), String> {
    audio::play_audio(&player, &path, volume).map_err(|e| format!("Failed to play audio: {e}"))
}

/// Tauri command to play an audio file (macOS stub)
#[cfg(target_os = "macos")]
#[tauri::command]
pub fn play_audio(path: String, volume: f32) -> Result<(), String> {
    audio::play_audio(&path, volume).map_err(|e| format!("Failed to play audio: {e}"))
}

/// Tauri command to play a builtin audio resource (non-macOS)
#[cfg(not(target_os = "macos"))]
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn play_builtin_audio(
    app: AppHandle,
    player: State<'_, AudioPlayerState>,
    resource_name: String,
    volume: f32,
) -> Result<(), String> {
    let resource_path = resolve_builtin_audio_path(&app, &resource_name)?;
    audio::play_audio(&player, &resource_path, volume)
        .map_err(|e| format!("Failed to play builtin audio: {e}"))
}

/// Tauri command to play a builtin audio resource (macOS stub)
#[cfg(target_os = "macos")]
#[tauri::command]
pub fn play_builtin_audio(
    app: AppHandle,
    resource_name: String,
    volume: f32,
) -> Result<(), String> {
    let resource_path = resolve_builtin_audio_path(&app, &resource_name)?;
    audio::play_audio(&resource_path, volume)
        .map_err(|e| format!("Failed to play builtin audio: {e}"))
}

/// Tauri command to stop audio playback (non-macOS)
#[cfg(not(target_os = "macos"))]
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn stop_audio(player: State<'_, AudioPlayerState>) -> Result<(), String> {
    audio::stop_audio(&player).map_err(|e| format!("Failed to stop audio: {e}"))
}

/// Tauri command to stop audio playback (macOS stub)
#[cfg(target_os = "macos")]
#[tauri::command]
pub fn stop_audio() -> Result<(), String> {
    audio::stop_audio().map_err(|e| format!("Failed to stop audio: {e}"))
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
