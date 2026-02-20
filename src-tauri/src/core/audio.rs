//! Audio playback functionality for break notifications.
//!
//! This module provides audio playback capabilities using the `rodio` crate.

mod models;
mod player;

pub use models::{AudioSettings, AudioSource};
pub use player::{AudioPlayer, PlaybackError};

use parking_lot::Mutex;
use std::sync::Arc;

/// Audio player state managed by Tauri.
pub type AudioPlayerState = Arc<Mutex<Option<AudioPlayer>>>;

/// Initializes the audio player and stores it in Tauri state.
///
/// # Errors
///
/// Returns an error if initializing the audio device or output stream fails.
pub fn init_audio_player() -> Result<AudioPlayerState, PlaybackError> {
    let player = AudioPlayer::new()?;
    tracing::info!("Audio player initialized successfully");
    Ok(Arc::new(Mutex::new(Some(player))))
}

/// Plays audio from a file path.
///
/// # Errors
///
/// Returns an error if:
/// - The audio player is not initialized
/// - Loading or decoding the audio file fails
/// - Starting playback fails
pub fn play_audio(
    player_state: &AudioPlayerState,
    path: &str,
    volume: f32,
) -> Result<(), PlaybackError> {
    player_state
        .lock()
        .as_mut()
        .ok_or(PlaybackError::NotInitialized)?
        .play(path, volume)?;
    tracing::debug!("Playing audio: {path} at volume {volume}");
    Ok(())
}

/// Stops currently playing audio.
///
/// # Errors
///
/// Returns an error if the audio player is not initialized.
pub fn stop_audio(player_state: &AudioPlayerState) -> Result<(), PlaybackError> {
    player_state
        .lock()
        .as_mut()
        .ok_or(PlaybackError::NotInitialized)?
        .stop();
    tracing::debug!("Audio playback stopped");
    Ok(())
}
