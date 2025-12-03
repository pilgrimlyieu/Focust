use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, thiserror::Error)]
pub enum PlaybackError {
    /// Failed to initialize audio output
    #[error("Failed to initialize audio output: {0}")]
    OutputStreamError(String),

    /// Audio player not initialized
    #[error("Audio player not initialized")]
    NotInitialized,

    /// Invalid volume value
    #[error("Invalid volume value: {0}. Volume must be between 0.0 and 1.0")]
    InvalidVolume(f32),

    /// Failed to open audio file
    #[error("Failed to open audio file: {0}")]
    FileError(#[from] std::io::Error),

    /// Failed to decode audio file
    #[error("Failed to decode audio file: {0}")]
    DecoderError(String),

    /// General playback failure
    #[error("Failed to play audio: {0}")]
    PlaybackFailed(String),
}

/// Audio player using rodio
pub struct AudioPlayer {
    /// Output stream (must be kept alive)
    _stream: OutputStream,
    /// Current sink for audio playback
    sink: Sink,
    /// Current volume (0.0 to 1.0)
    current_volume: f32,
}

impl AudioPlayer {
    /// Creates a new audio player.
    ///
    /// # Errors
    ///
    /// Returns an error if initializing the default output stream fails.
    pub fn new() -> Result<Self, PlaybackError> {
        // Initialize output stream using rodio 0.21 API
        let stream_handle = OutputStreamBuilder::open_default_stream()
            .map_err(|e| PlaybackError::OutputStreamError(e.to_string()))?;

        // Create a sink connected to the mixer
        let sink = Sink::connect_new(stream_handle.mixer());

        tracing::info!("Audio player initialized successfully");

        Ok(Self {
            _stream: stream_handle,
            sink,
            current_volume: 0.6,
        })
    }

    /// Plays an audio file at the specified volume.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The volume is not between 0.0 and 1.0
    /// - The file does not exist
    /// - Opening or decoding the audio file fails
    pub fn play(&mut self, path: &str, volume: f32) -> Result<(), PlaybackError> {
        // Validate volume
        if !(0.0..=1.0).contains(&volume) {
            return Err(PlaybackError::InvalidVolume(volume));
        }

        // Validate file exists before attempting to open
        if !std::path::Path::new(path).exists() {
            return Err(PlaybackError::FileError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Audio file not found: {path}"),
            )));
        }

        // Stop any currently playing audio before starting new playback
        // This ensures clean state and prevents resource conflicts
        if !self.sink.empty() {
            tracing::debug!("Stopping current audio before playing new file");
            self.stop();
        }

        // Open the audio file
        tracing::debug!("Opening audio file: {path}");
        let file = File::open(path).map_err(|e| {
            tracing::error!("Failed to open audio file {path}: {e}");
            e
        })?;

        // Decode the audio file
        tracing::debug!("Decoding audio file: {path}");
        let source = Decoder::new(BufReader::new(file)).map_err(|e| {
            let err_msg = format!("Failed to decode audio file {path}: {e}");
            tracing::error!("{err_msg}");
            PlaybackError::DecoderError(err_msg)
        })?;

        // Set volume and append source
        self.sink.set_volume(volume);
        self.sink.append(source);

        // Sink plays automatically after appending
        self.current_volume = volume;

        tracing::debug!("Audio playback started: {path} at volume {volume}");
        Ok(())
    }

    /// Stop the currently playing audio
    pub fn stop(&mut self) {
        self.sink.stop();
        tracing::debug!("Audio playback stopped");
    }

    /// Pause the currently playing audio
    pub fn pause(&mut self) {
        self.sink.pause();
        tracing::debug!("Audio playback paused");
    }

    /// Resume the paused audio
    pub fn resume(&mut self) {
        self.sink.play();
        tracing::debug!("Audio playback resumed");
    }

    /// Check if audio is currently playing
    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused() && !self.sink.empty()
    }

    /// Get current volume
    pub fn volume(&self) -> f32 {
        self.current_volume
    }

    /// Sets the playback volume.
    ///
    /// # Errors
    ///
    /// Returns an error if the volume is not between 0.0 and 1.0.
    pub fn set_volume(&mut self, volume: f32) -> Result<(), PlaybackError> {
        if !(0.0..=1.0).contains(&volume) {
            return Err(PlaybackError::InvalidVolume(volume));
        }

        self.current_volume = volume;
        self.sink.set_volume(volume);
        tracing::debug!("Volume set to {volume}");

        Ok(())
    }
}

#[cfg(test)]
#[expect(clippy::float_cmp)]
mod tests {
    use super::{AudioPlayer, PlaybackError};
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper to create a temporary WAV file for testing
    fn create_test_audio_file(dir: &TempDir, name: &str) -> String {
        let file_path = dir.path().join(name);
        let mut file = File::create(&file_path).expect("Failed to create test file");

        // Minimal valid WAV file (44 bytes header + 1 sample)
        // RIFF header
        file.write_all(b"RIFF").unwrap();
        file.write_all(&36u32.to_le_bytes()).unwrap(); // chunk size
        file.write_all(b"WAVE").unwrap();

        // fmt subchunk
        file.write_all(b"fmt ").unwrap();
        file.write_all(&16u32.to_le_bytes()).unwrap(); // subchunk1 size
        file.write_all(&1u16.to_le_bytes()).unwrap(); // audio format (PCM)
        file.write_all(&1u16.to_le_bytes()).unwrap(); // num channels
        file.write_all(&44100u32.to_le_bytes()).unwrap(); // sample rate
        file.write_all(&88200u32.to_le_bytes()).unwrap(); // byte rate
        file.write_all(&2u16.to_le_bytes()).unwrap(); // block align
        file.write_all(&16u16.to_le_bytes()).unwrap(); // bits per sample

        // data subchunk
        file.write_all(b"data").unwrap();
        file.write_all(&4u32.to_le_bytes()).unwrap(); // subchunk2 size
        file.write_all(&[0u8, 0, 0, 0]).unwrap(); // sample data

        file_path.to_string_lossy().to_string()
    }

    #[test]
    fn audio_player_creation() {
        // AudioPlayer::new() requires actual audio hardware
        match AudioPlayer::new() {
            Ok(player) => {
                assert_eq!(player.volume(), 0.6);
                assert!(!player.is_playing());
            }
            Err(PlaybackError::OutputStreamError(_)) => {
                eprintln!("Audio hardware not available, skipping test");
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }

    #[test]
    fn invalid_volume() {
        if let Ok(mut player) = AudioPlayer::new() {
            assert!(player.set_volume(-0.1).is_err());
            assert!(player.set_volume(1.1).is_err());
            player.set_volume(0.0).unwrap();
            player.set_volume(1.0).unwrap();
            player.set_volume(0.5).unwrap();
        }
    }

    #[test]
    fn play_nonexistent_file() {
        if let Ok(mut player) = AudioPlayer::new() {
            let result = player.play("/nonexistent/file.mp3", 0.5);
            assert!(result.is_err());
            match result {
                Err(PlaybackError::FileError(_)) => {
                    // Expected error
                }
                _ => panic!("Expected FileError for nonexistent file"),
            }
        }
    }

    #[test]
    fn play_invalid_audio_file() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_file = temp_dir.path().join("invalid.mp3");
        std::fs::write(&invalid_file, b"not a valid audio file").unwrap();

        if let Ok(mut player) = AudioPlayer::new() {
            let result = player.play(&invalid_file.to_string_lossy(), 0.5);
            assert!(result.is_err());
            match result {
                Err(PlaybackError::DecoderError(_)) => {
                    // Expected error
                }
                _ => panic!("Expected DecoderError for invalid audio file"),
            }
        }
    }

    #[test]
    fn play_valid_audio_file() {
        let temp_dir = TempDir::new().unwrap();
        let audio_file = create_test_audio_file(&temp_dir, "test.wav");

        if let Ok(mut player) = AudioPlayer::new() {
            let result = player.play(&audio_file, 0.5);
            if result.is_err() {
                eprintln!("Audio playback not available in test environment");
            }
        }
    }

    #[test]
    fn stop_when_not_playing() {
        if let Ok(mut player) = AudioPlayer::new() {
            player.stop();
            assert!(!player.is_playing());
        }
    }

    #[test]
    fn volume_persistence() {
        if let Ok(mut player) = AudioPlayer::new() {
            player.set_volume(0.3).unwrap();
            assert_eq!(player.volume(), 0.3);

            player.set_volume(0.8).unwrap();
            assert_eq!(player.volume(), 0.8);
        }
    }

    #[test]
    fn play_with_invalid_volume() {
        let temp_dir = TempDir::new().unwrap();
        let audio_file = create_test_audio_file(&temp_dir, "test.wav");

        if let Ok(mut player) = AudioPlayer::new() {
            let result = player.play(&audio_file, 1.5);
            assert!(result.is_err());
            match result {
                Err(PlaybackError::InvalidVolume(v)) => {
                    assert_eq!(v, 1.5);
                }
                _ => panic!("Expected InvalidVolume error"),
            }
        }
    }

    #[test]
    fn sequential_playback() {
        let temp_dir = TempDir::new().unwrap();
        let audio_file1 = create_test_audio_file(&temp_dir, "test1.wav");
        let audio_file2 = create_test_audio_file(&temp_dir, "test2.wav");

        if let Ok(mut player) = AudioPlayer::new()
            && player.play(&audio_file1, 0.5).is_ok()
        {
            let result = player.play(&audio_file2, 0.6);
            if result.is_err() {
                eprintln!("Audio playback not available in test environment");
            } else {
                assert_eq!(player.volume(), 0.6);
            }
        }
    }
}
