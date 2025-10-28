use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Default, Clone, Debug, TS)]
#[ts(export)]
#[serde(tag = "source")]
pub enum AudioSource {
    /// No audio
    #[default]
    None,
    /// Builtin audio resource
    #[serde(rename = "Builtin")]
    Builtin { name: String },
    /// File path audio source
    #[serde(rename = "FilePath")]
    FilePath { path: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct AudioSettings {
    #[serde(flatten)]
    pub source: AudioSource,
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
    pub fn get_path(&self) -> Option<String> {
        match &self.source {
            AudioSource::None => None,
            AudioSource::Builtin { name } => {
                // Return resource identifier in format: "sounds/{name}.mp3"
                // This will be resolved to actual path at runtime using AppHandle::path().resource_dir()
                Some(format!("sounds/{name}.mp3"))
            }
            AudioSource::FilePath { path } => Some(path.clone()),
        }
    }

    /// Check if this audio source is a builtin resource
    pub fn is_builtin(&self) -> bool {
        matches!(self.source, AudioSource::Builtin { .. })
    }
}
