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

/// Audio player state managed by Tauri (non-macOS only)
#[cfg(not(target_os = "macos"))]
pub type AudioPlayerState = Arc<Mutex<Option<AudioPlayer>>>;

/// Initialize the audio player and store it in Tauri state (non-macOS)
#[cfg(not(target_os = "macos"))]
pub fn init_audio_player() -> Result<AudioPlayerState, PlaybackError> {
    let player = AudioPlayer::new()?;
    tracing::info!("Audio player initialized successfully");
    Ok(Arc::new(Mutex::new(Some(player))))
}

/// Initialize audio (macOS stub - audio not supported)
#[cfg(target_os = "macos")]
pub fn init_audio_player() -> Result<(), PlaybackError> {
    tracing::warn!("Audio playback is not supported on macOS due to CoreAudio backend limitations");
    Ok(())
}

/// Play audio from a file path (non-macOS)
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

/// Play audio (macOS stub)
#[cfg(target_os = "macos")]
pub fn play_audio(_path: &str, _volume: f32) -> Result<(), PlaybackError> {
    Err(PlaybackError::PlaybackFailed(
        "Audio playback is not supported on macOS".to_string(),
    ))
}

/// Stop currently playing audio (non-macOS)
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

/// Stop audio (macOS stub)
#[cfg(target_os = "macos")]
pub fn stop_audio() -> Result<(), PlaybackError> {
    Err(PlaybackError::PlaybackFailed(
        "Audio playback is not supported on macOS".to_string(),
    ))
}
