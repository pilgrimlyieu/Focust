use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub enum AudioSource {
    None,             // Disabled audio
    Provided(String), // Predefined audio source
    Custom(String),   // Custom audio file path
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(rename_all = "camelCase")]
pub struct AudioSettings {
    pub source: AudioSource, // Audio source type, e.g., Provided, Custom(Path)
    pub volume: u8,          // Volume level from 0 to 100
}

impl Default for AudioSettings {
    fn default() -> Self {
        AudioSettings {
            source: AudioSource::None,
            volume: 50, // Default volume set to 50%
        }
    }
}
