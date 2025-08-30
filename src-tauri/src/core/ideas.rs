use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub enum IdeasSource {
    None,
    Provided,
    Custom,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(rename_all = "camelCase")]
pub struct IdeasSettings {
    /// Ideas source
    pub source: IdeasSource,
}

impl Default for IdeasSettings {
    fn default() -> Self {
        IdeasSettings {
            source: IdeasSource::Provided, // Default to provided ideas source
        }
    }
}
