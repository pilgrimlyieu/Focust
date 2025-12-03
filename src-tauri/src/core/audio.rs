//! Audio playback functionality for break notifications.
//!
//! This module provides audio playback capabilities using the `rodio` crate.
//!
//! # Platform Support
//!
//! **Note**: Audio is temporarily disabled on macOS due to CoreAudio backend
//! limitations in `rodio`/`cpal`. The CoreAudio backend doesn't implement `Send`,
//! making it incompatible with Tauri's state management which requires `Send + Sync`.
//!
//! **Status**: Waiting for cpal 0.17.0+ which includes
//! [PR #1021](https://github.com/RustAudio/cpal/pull/1021) that fixes the Send trait issue.
//!
//! **Workarounds considered**:
//! - Using `nsound` instead of `rodio` (macOS-specific, adds complexity)
//! - Thread-local audio player (incompatible with Tauri's async model)
//!
//! **Resolution**: Once cpal 0.17.0+ is released, we can re-enable full audio
//! support on macOS by simply removing the `#[cfg(not(target_os = "macos"))]` guards.

mod models;
mod player;

pub use models::{AudioSettings, AudioSource};
pub use player::PlaybackError;

// Audio is only supported on non-macOS platforms
// macOS CoreAudio backend in rodio/cpal doesn't implement Send, making it incompatible with Tauri's state management
#[cfg(not(target_os = "macos"))]
pub use player::AudioPlayer;

#[cfg(not(target_os = "macos"))]
use parking_lot::Mutex;
#[cfg(not(target_os = "macos"))]
use std::sync::Arc;

/// Audio player state managed by Tauri (non-macOS only).
#[cfg(not(target_os = "macos"))]
pub type AudioPlayerState = Arc<Mutex<Option<AudioPlayer>>>;

/// Initializes the audio player and stores it in Tauri state (non-macOS).
///
/// # Errors
///
/// Returns an error if initializing the audio device or output stream fails.
#[cfg(not(target_os = "macos"))]
pub fn init_audio_player() -> Result<AudioPlayerState, PlaybackError> {
    let player = AudioPlayer::new()?;
    tracing::info!("Audio player initialized successfully");
    Ok(Arc::new(Mutex::new(Some(player))))
}

/// Initializes audio (macOS stub - audio not supported).
///
/// # Errors
///
/// This function does not return errors in normal operation (always returns `Ok(())`).
#[cfg(target_os = "macos")]
pub fn init_audio_player() -> Result<(), PlaybackError> {
    tracing::warn!("Audio playback is not supported on macOS due to CoreAudio backend limitations");
    Ok(())
}

/// Plays audio from a file path (non-macOS).
///
/// # Errors
///
/// Returns an error if:
/// - The audio player is not initialized
/// - Loading or decoding the audio file fails
/// - Starting playback fails
#[cfg(not(target_os = "macos"))]
pub fn play_audio(
    player_state: &AudioPlayerState,
    path: &str,
    volume: f32,
) -> Result<(), PlaybackError> {
    let mut player_guard = player_state.lock();

    if let Some(ref mut player) = *player_guard {
        player.play(path, volume)?;
        tracing::debug!("Playing audio: {path} at volume {volume}");
        Ok(())
    } else {
        Err(PlaybackError::NotInitialized)
    }
}

/// Plays audio (macOS stub).
///
/// # Errors
///
/// Always returns an error indicating audio is not supported on macOS.
#[cfg(target_os = "macos")]
pub fn play_audio(_path: &str, _volume: f32) -> Result<(), PlaybackError> {
    Err(PlaybackError::PlaybackFailed(
        "Audio playback is not supported on macOS".to_string(),
    ))
}

/// Stops currently playing audio (non-macOS).
///
/// # Errors
///
/// Returns an error if the audio player is not initialized.
#[cfg(not(target_os = "macos"))]
pub fn stop_audio(player_state: &AudioPlayerState) -> Result<(), PlaybackError> {
    let mut player_guard = player_state.lock();

    if let Some(ref mut player) = *player_guard {
        player.stop();
        tracing::debug!("Audio playback stopped");
        Ok(())
    } else {
        Err(PlaybackError::NotInitialized)
    }
}

/// Stops audio (macOS stub).
///
/// # Errors
///
/// Always returns an error indicating audio is not supported on macOS.
#[cfg(target_os = "macos")]
pub fn stop_audio() -> Result<(), PlaybackError> {
    Err(PlaybackError::PlaybackFailed(
        "Audio playback is not supported on macOS".to_string(),
    ))
}
