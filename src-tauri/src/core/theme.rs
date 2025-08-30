use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct FontFamily(String);

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct HexColor(String);

impl HexColor {
    pub fn is_valid(&self) -> bool {
        self.0.starts_with('#')
            && self.0.len() == 7
            && self.0[1..].chars().all(|c| c.is_ascii_hexdigit())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub enum BackgroundSource {
    Solid(String),
    ImagePath(String),
    ImageFolder(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(rename_all = "camelCase")]
pub struct ThemeSettings {
    /// Unique identifier for the break settings
    pub background: BackgroundSource, 
    /// If the break is enabled
    pub text_color: HexColor,         
    /// Theme settings for the break
    pub blur_radius: u8,              
    /// Audio settings for the break
    pub opacity: f32,                 
    /// If the break should be shown in fullscreen
    pub font_size: u8,                
    /// Ideas settings for the break
    pub font_family: FontFamily,      
}

impl Default for ThemeSettings {
    fn default() -> Self {
        ThemeSettings {
            background: BackgroundSource::Solid("#000000".to_string()),
            text_color: HexColor("#FFFFFF".to_string()),
            blur_radius: 0,
            opacity: 1.0,
            font_size: 16,
            font_family: FontFamily("Arial".to_string()),
        }
    }
}
