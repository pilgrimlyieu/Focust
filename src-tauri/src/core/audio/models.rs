use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Audio source configuration for break sounds
#[derive(Serialize, Deserialize, Default, Clone, Debug, TS)]
#[serde(tag = "source", rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub enum AudioSource {
    /// No audio
    #[default]
    None, // No audio source

    /// Builtin audio resource
    Builtin { name: String }, // Builtin audio resource name

    /// File path audio source
    FilePath { path: String }, // File path to audio source
}

/// Audio settings for break sounds
#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, rename_all = "camelCase")]
pub struct AudioSettings {
    /// Audio source configuration
    #[serde(flatten)]
    pub source: AudioSource,
    /// Volume level (0.0 to 1.0)
    pub volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            source: AudioSource::None,
            volume: 0.6,
        }
    }
}

impl AudioSettings {
    /// Get the file path for the audio source
    /// Returns None if the source is None
    /// For builtin sources, returns the resource identifier that can be resolved at runtime
    /// For file paths, returns the absolute path
    #[must_use]
    pub fn get_path(&self) -> Option<String> {
        match &self.source {
            AudioSource::None => None,
            AudioSource::Builtin { name } => {
                // Return resource identifier in format: "sounds/{name}.mp3"
                Some(format!("sounds/{name}.mp3"))
            }
            AudioSource::FilePath { path } => Some(path.clone()),
        }
    }

    /// Check if this audio source is a builtin resource
    #[must_use]
    pub fn is_builtin(&self) -> bool {
        matches!(self.source, AudioSource::Builtin { .. })
    }
}
