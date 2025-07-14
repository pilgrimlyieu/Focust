use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IdeasSource {
    None,
    Provided,
    Custom,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdeasSettings {
    pub source: IdeasSource, // Source of ideas, e.g., None, Provided, Custom
}

impl Default for IdeasSettings {
    fn default() -> Self {
        IdeasSettings {
            source: IdeasSource::Provided, // Default to provided ideas source
        }
    }
}
