use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub enum AudioSource {
    /// Disable audio
    None,
    /// Predefined audio source
    Provided(String),
    /// Custom audio file path
    Custom(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(rename_all = "camelCase")]
pub struct AudioSettings {
    /// Audio source
    pub source: AudioSource,
    /// Volume level (0-100)
    pub volume: u8,
}

impl Default for AudioSettings {
    fn default() -> Self {
        AudioSettings {
            source: AudioSource::None,
            volume: 50, // Default volume set to 50%
        }
    }
}
