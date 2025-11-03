mod models;
mod player;

pub use models::{AudioSettings, AudioSource};
pub use player::{AudioPlayer, PlaybackError};

use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;

/// Global audio player instance
static AUDIO_PLAYER: LazyLock<Arc<Mutex<Option<AudioPlayer>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

/// Initialize the global audio player
pub async fn init_audio_player() -> Result<(), PlaybackError> {
    let mut player_guard = AUDIO_PLAYER.lock().await;
    if player_guard.is_none() {
        let player = AudioPlayer::new()?;
        *player_guard = Some(player);
        tracing::info!("Audio player initialized successfully");
    }
    Ok(())
}

/// Get the global audio player instance
pub fn get_audio_player() -> Arc<Mutex<Option<AudioPlayer>>> {
    AUDIO_PLAYER.clone()
}

/// Play audio from a file path
pub async fn play_audio(path: &str, volume: f32) -> Result<(), PlaybackError> {
    let player_arc = AUDIO_PLAYER.clone();
    let mut player_guard = player_arc.lock().await;

    if let Some(ref mut player) = *player_guard {
        player.play(path, volume)?;
        tracing::debug!("Playing audio: {path} at volume {volume}");
        Ok(())
    } else {
        Err(PlaybackError::NotInitialized)
    }
}

/// Stop currently playing audio
pub async fn stop_audio() -> Result<(), PlaybackError> {
    let player_arc = AUDIO_PLAYER.clone();
    let mut player_guard = player_arc.lock().await;

    if let Some(ref mut player) = *player_guard {
        player.stop();
        tracing::debug!("Audio playback stopped");
        Ok(())
    } else {
        Err(PlaybackError::NotInitialized)
    }
}
