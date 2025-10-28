use crate::core::audio;
use std::path::PathBuf;
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

/// Helper function to resolve builtin audio resource path
fn resolve_builtin_audio_path(app: &AppHandle, resource_name: &str) -> Result<String, String> {
    // TODO: refactor
    // Try to get the resource directory (works in production)
    let resource_dir_result = app.path().resource_dir();

    let audio_path: PathBuf = if let Ok(resource_dir) = resource_dir_result {
        // Production: use bundled resources
        resource_dir
            .join("sounds")
            .join(format!("{}.mp3", resource_name))
    } else {
        // Development: use source directory
        // Get the app directory
        let app_dir = app
            .path()
            .app_config_dir()
            .map_err(|e| format!("Failed to get app directory: {e}"))?;

        // Go up to project root and into assets
        app_dir
            .parent()
            .and_then(|p| p.parent())
            .ok_or_else(|| "Failed to resolve project root".to_string())?
            .join("src-tauri")
            .join("assets")
            .join("sounds")
            .join(format!("{resource_name}.mp3"))
    };

    tracing::debug!(
        "Resolving builtin audio: {resource_name} -> {}",
        audio_path.display()
    );

    // Check if file exists
    if !audio_path.exists() {
        // Try alternative paths for development
        let cwd =
            std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

        tracing::debug!("Current working directory: {}", cwd.display());

        // Path 1: Assume cwd is src-tauri directory
        let dev_path1 = cwd
            .join("assets")
            .join("sounds")
            .join(format!("{resource_name}.mp3"));

        tracing::debug!("Trying dev path 1: {}", dev_path1.display());

        if dev_path1.exists() {
            return dev_path1
                .to_str()
                .ok_or_else(|| "Invalid path encoding".to_string())
                .map(|s| s.to_string());
        }

        // Path 2: Assume cwd is project root
        let dev_path2 = cwd
            .join("src-tauri")
            .join("assets")
            .join("sounds")
            .join(format!("{resource_name}.mp3"));

        tracing::debug!("Trying dev path 2: {}", dev_path2.display());

        if dev_path2.exists() {
            return dev_path2
                .to_str()
                .ok_or_else(|| "Invalid path encoding".to_string())
                .map(|s| s.to_string());
        }

        return Err(format!(
            "Builtin audio resource '{resource_name}' not found. Tried:\n  1. {}\n  2. {}\n  3. {}",
            audio_path.display(),
            dev_path1.display(),
            dev_path2.display()
        ));
    }

    // Convert to string
    audio_path
        .to_str()
        .ok_or_else(|| "Invalid path encoding".to_string())
        .map(|s| s.to_string())
}
