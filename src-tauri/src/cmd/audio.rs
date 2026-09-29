//! Tauri commands for audio playback control.
//!
//! This module provides commands to play and stop audio files, including
//! both custom audio files and built-in sound resources.

use anyhow::{Context, anyhow};
use tauri::State;
use tauri::async_runtime::spawn_blocking;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

use crate::core::audio;
use crate::core::audio::AudioPlayerState;

/// Plays an audio file at the specified path with the given volume.
///
/// # Errors
///
/// Returns an error if:
/// - The audio file cannot be loaded or decoded
/// - The audio device fails to initialize
/// - The playback fails to start
#[tauri::command]
pub async fn play_audio(
    player: State<'_, AudioPlayerState>,
    path: String,
    volume: f32,
) -> Result<(), String> {
    tracing::debug!("play_audio command called: path={path}, volume={volume}");

    // File I/O, decoding and the player mutex must not block async workers.
    let player = player.inner().clone();
    spawn_blocking(move || audio::play_audio(&player, &path, volume))
        .await
        .map_err(|e| format!("Audio task failed: {e}"))?
        .map_err(|e| {
            let error_msg = format!("Failed to play audio: {e}");
            tracing::error!("Audio playback error: {error_msg}");
            error_msg
        })
        .inspect(|_result| {
            tracing::debug!("play_audio command completed successfully");
        })
}

/// Plays a built-in audio resource by name with the given volume.
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

    let player = player.inner().clone();
    spawn_blocking(move || {
        let resource_path = resolve_builtin_audio_path(&app, &resource_name)?;
        audio::play_audio(&player, &resource_path, volume).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Audio task failed: {e}"))?
    .map_err(|e| {
        let error_msg = format!("Failed to play builtin audio: {e}");
        tracing::error!("Audio playback error: {error_msg}");
        error_msg
    })
    .inspect(|_result| {
        tracing::debug!("play_builtin_audio command completed successfully");
    })
}

/// Stops the currently playing audio.
///
/// # Errors
///
/// Returns an error if the audio player fails to stop playback.
#[tauri::command]
pub async fn stop_audio(player: State<'_, AudioPlayerState>) -> Result<(), String> {
    tracing::debug!("stop_audio command called");
    let player = player.inner().clone();
    spawn_blocking(move || audio::stop_audio(&player))
        .await
        .map_err(|e| format!("Audio task failed: {e}"))?
        .map_err(|e| {
            let error_msg = format!("Failed to stop audio: {e}");
            tracing::error!("Audio stop error: {error_msg}");
            error_msg
        })
        .inspect(|_result| {
            tracing::debug!("stop_audio command completed successfully");
        })
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
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{play_audio, stop_audio};
    use crate::core::audio::AudioPlayerState;
    use parking_lot::Mutex;
    use std::sync::{Arc, mpsc};
    use std::thread;
    use std::time::Duration;
    use tauri::Manager;
    use tauri::test::mock_app;
    use tokio::runtime::Builder;

    #[test]
    fn audio_commands_leave_runtime_responsive_while_waiting_for_lock() {
        for stop in [false, true] {
            let state: AudioPlayerState = Arc::new(Mutex::new(None));
            let guard = state.lock();
            let cloned = Arc::clone(&state);
            let (tx, rx) = mpsc::channel();
            let worker = thread::spawn(move || {
                let app = mock_app();
                app.manage(cloned);
                let runtime = Builder::new_current_thread().build().unwrap();
                runtime.block_on(async {
                    // Poll the command first. With an inline blocking lock, even
                    // the sibling future cannot run until the audio lock opens.
                    let (result, ()) = tokio::join!(biased;
                        async {
                            if stop {
                                stop_audio(app.state()).await
                            } else {
                                play_audio(app.state(), "unused.wav".into(), 0.6).await
                            }
                        },
                        async { tx.send(()).unwrap(); }
                    );
                    assert!(result.is_err()); // Uninitialized test player.
                });
            });
            let responsive = rx.recv_timeout(Duration::from_secs(1));
            drop(guard);
            worker.join().unwrap();
            assert!(
                responsive.is_ok(),
                "audio command blocked the async runtime (stop={stop})"
            );
        }
    }
}
