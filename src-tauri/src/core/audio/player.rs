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
    /// Create a new audio player
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

    /// Play an audio file at the specified volume
    pub fn play(&mut self, path: &str, volume: f32) -> Result<(), PlaybackError> {
        // Validate volume
        if !(0.0..=1.0).contains(&volume) {
            return Err(PlaybackError::InvalidVolume(volume));
        }

        // Stop any currently playing audio
        self.stop();

        // Open the audio file
        let file = File::open(path)?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| PlaybackError::DecoderError(e.to_string()))?;

        // Set volume and append source
        self.sink.set_volume(volume);
        self.sink.append(source);

        // Sink plays automatically after appending
        self.current_volume = volume;

        tracing::debug!("Playing audio: {path} at volume {volume}");
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

    /// Set volume (0.0 to 1.0)
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
