//! Tauri commands for audio playback control.
//!
//! This module provides commands to play and stop audio files, including
//! both custom audio files and built-in sound resources.
//!
//! # Platform Support
//!
//! **Note**: Audio is temporarily disabled on macOS due to cpal Send trait limitations.
//! See `src/core/audio.rs` for detailed explanation and restoration plan.
//! Expected to be resolved in cpal 0.17.0+.

#[cfg(not(target_os = "macos"))]
use tauri::State;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::core::audio;
#[cfg(not(target_os = "macos"))]
use crate::core::audio::AudioPlayerState;
#[cfg(target_os = "macos")]
use crate::tauri_error;

/// Plays an audio file at the specified path with the given volume (non-macOS).
///
/// # Errors
///
/// Returns an error if:
/// - The audio file cannot be loaded or decoded
/// - The audio device fails to initialize
/// - The playback fails to start
#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn play_audio(
    player: State<'_, AudioPlayerState>,
    path: String,
    volume: f32,
) -> Result<(), String> {
    tracing::debug!("play_audio command called: path={path}, volume={volume}");

    audio::play_audio(&player, &path, volume)
        .map_err(|e| {
            let error_msg = format!("Failed to play audio: {e}");
            tracing::error!("Audio playback error: {error_msg}");
            error_msg
        })
        .inspect(|_result| {
            tracing::debug!("play_audio command completed successfully");
        })
}

/// Plays an audio file at the specified path with the given volume (macOS stub).
///
/// # Errors
///
/// Returns an error if:
/// - The audio file cannot be loaded or decoded
/// - The audio device fails to initialize
/// - The playback fails to start
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn play_audio(path: String, volume: f32) -> Result<(), String> {
    tauri_error!(audio::play_audio(&path, volume), "Failed to play audio")
}

/// Plays a built-in audio resource by name with the given volume (non-macOS).
///
/// The resource is resolved from the `assets/sounds/` directory.
///
/// # Errors
///
/// Returns an error if:
/// - The resource path cannot be resolved
/// - The built-in audio file is not found
/// - The audio file cannot be loaded or decoded
/// - The audio device fails to initialize
/// - The playback fails to start
#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn play_builtin_audio(
    app: AppHandle,
    player: State<'_, AudioPlayerState>,
    resource_name: String,
    volume: f32,
) -> Result<(), String> {
    tracing::debug!(
        "play_builtin_audio command called: resource_name={resource_name}, volume={volume}"
    );

    let resource_path = resolve_builtin_audio_path(&app, &resource_name)?;

    audio::play_audio(&player, &resource_path, volume)
        .map_err(|e| {
            let error_msg = format!("Failed to play builtin audio: {e}");
            tracing::error!("Audio playback error: {error_msg}");
            error_msg
        })
        .inspect(|_result| {
            tracing::debug!("play_builtin_audio command completed successfully");
        })
}

/// Plays a built-in audio resource by name with the given volume (macOS stub).
///
/// The resource is resolved from the `assets/sounds/` directory.
///
/// # Errors
///
/// Returns an error if:
/// - The resource path cannot be resolved
/// - The built-in audio file is not found
/// - The audio file cannot be loaded or decoded
/// - The audio device fails to initialize
/// - The playback fails to start
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn play_builtin_audio(
    app: AppHandle,
    resource_name: String,
    volume: f32,
) -> Result<(), String> {
    let resource_path = resolve_builtin_audio_path(&app, &resource_name)?;
    tauri_error!(
        audio::play_audio(&resource_path, volume),
        "Failed to play builtin audio"
    )
}

/// Stops the currently playing audio (non-macOS).
///
/// # Errors
///
/// Returns an error if the audio player fails to stop playback.
#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn stop_audio(player: State<'_, AudioPlayerState>) -> Result<(), String> {
    tracing::debug!("stop_audio command called");
    audio::stop_audio(&player)
        .map_err(|e| {
            let error_msg = format!("Failed to stop audio: {e}");
            tracing::error!("Audio stop error: {error_msg}");
            error_msg
        })
        .inspect(|_result| {
            tracing::debug!("stop_audio command completed successfully");
        })
}

/// Stops the currently playing audio (macOS stub).
///
/// # Errors
///
/// Returns an error if the audio player fails to stop playback.
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn stop_audio() -> Result<(), String> {
    tauri_error!(audio::stop_audio(), "Failed to stop audio")
}

/// Resolves the absolute path of a built-in audio resource.
///
/// Looks up the resource in the `assets/sounds/` directory and validates
/// that it exists before returning the path.
///
/// # Errors
///
/// Returns an error if:
/// - The resource path cannot be resolved
/// - The resource file does not exist at the resolved path
/// - The path contains invalid UTF-8 encoding
fn resolve_builtin_audio_path(app: &AppHandle, resource_name: &str) -> Result<String, String> {
    use anyhow::{Context, anyhow};

    let resource_relative_path = format!("assets/sounds/{resource_name}.mp3");

    tracing::debug!("Attempting to resolve builtin audio resource: {resource_relative_path}");

    let resolved_path_buf = app
        .path()
        .resolve(&resource_relative_path, BaseDirectory::Resource)
        .with_context(|| format!("Failed to resolve resource path for '{resource_name}'"))
        .map_err(|e| e.to_string())?;

    tracing::debug!(
        "Resolved builtin audio path: {}",
        resolved_path_buf.display()
    );

    if !resolved_path_buf.exists() {
        return Err(anyhow!(
            "Builtin audio resource '{}' not found at resolved path: {}",
            resource_name,
            resolved_path_buf.display()
        )
        .to_string());
    }

    resolved_path_buf
        .to_str()
        .ok_or_else(|| anyhow!("Invalid path encoding for resource '{resource_name}'").to_string())
        .map(std::string::ToString::to_string)
}
