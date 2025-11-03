use std::{fmt::Display, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct FontFamily(String);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, TS)]
pub struct HexColor(String);

impl HexColor {
    #[must_use]
    fn is_valid(s: &str) -> bool {
        s.starts_with('#') && s.len() == 7 && s[1..].chars().all(|c| c.is_ascii_hexdigit())
    }

    #[must_use]
    pub fn new(s: &str) -> Self {
        HexColor::from_str(s).unwrap_or_default()
    }
}

impl FromStr for HexColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if Self::is_valid(s) {
            Ok(HexColor(s.to_string()))
        } else {
            Err(())
        }
    }
}

impl Default for HexColor {
    fn default() -> Self {
        HexColor("#FFFFFF".to_string())
    }
}

impl Display for HexColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Resolved background for break window
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
pub struct ResolvedBackground {
    pub kind: BackgroundKind,
    pub value: String,
}

impl Default for ResolvedBackground {
    fn default() -> Self {
        Self {
            kind: BackgroundKind::Solid,
            value: HexColor::default().to_string(),
        }
    }
}

impl ResolvedBackground {
    #[must_use]
    pub fn new_solid(color: String) -> Self {
        Self {
            kind: BackgroundKind::Solid,
            value: color,
        }
    }

    #[must_use]
    pub fn new_image(path: String) -> Self {
        Self {
            kind: BackgroundKind::Image,
            value: path,
        }
    }

    #[must_use]
    pub fn is_solid(&self) -> bool {
        self.kind == BackgroundKind::Solid
    }

    #[must_use]
    pub fn is_image(&self) -> bool {
        self.kind == BackgroundKind::Image
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
#[derive(Default)]
pub enum BackgroundKind {
    #[default]
    Solid,
    Image,
}

impl Display for BackgroundKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind_str = match self {
            BackgroundKind::Solid => "solid",
            BackgroundKind::Image => "image",
        };
        write!(f, "{kind_str}")
    }
}
impl Deref for FontFamily {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum BackgroundSource {
    /// Solid color background (hex color code)
    Solid(HexColor),
    /// Image background from a specific file path
    ImagePath(String),
    /// Image background from a folder (randomly selected)
    ImageFolder(String),
}

/// Theme settings for break windows
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ThemeSettings {
    /// Background source for the break window (solid color, image path, or image folder)
    pub background: BackgroundSource,
    /// Text color in hex format
    pub text_color: HexColor,
    /// Blur radius for background effect in pixels
    pub blur_radius: u8,
    /// Opacity of the background (0.0 - 1.0)
    pub opacity: f32,
    /// Font size in pixels
    pub font_size: u8,
    /// Font family name
    pub font_family: FontFamily,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        ThemeSettings {
            background: BackgroundSource::Solid(HexColor::new("#1f2937")),
            text_color: HexColor::new("#f8fafc"),
            blur_radius: 8,
            opacity: 0.9,
            font_size: 24,
            font_family: FontFamily("Arial".to_string()),
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn test_background_source_solid() {
        let bg = BackgroundSource::Solid(HexColor::new("#000000"));
        match bg {
            BackgroundSource::Solid(color) => assert_eq!(color.to_string(), "#000000"),
            _ => panic!("Expected Solid variant"),
        }
    }

    #[test]
    fn test_background_source_image_path() {
        let bg = BackgroundSource::ImagePath("/path/to/image.jpg".to_string());
        match bg {
            BackgroundSource::ImagePath(path) => assert_eq!(path, "/path/to/image.jpg"),
            _ => panic!("Expected ImagePath variant"),
        }
    }

    #[test]
    fn test_background_source_image_folder() {
        let bg = BackgroundSource::ImageFolder("/path/to/folder".to_string());
        match bg {
            BackgroundSource::ImageFolder(folder) => assert_eq!(folder, "/path/to/folder"),
            _ => panic!("Expected ImageFolder variant"),
        }
    }

    #[test]
    fn test_background_source_clone() {
        let bg = BackgroundSource::Solid(HexColor::new("#123456"));
        let cloned = bg.clone();
        match (bg, cloned) {
            (BackgroundSource::Solid(a), BackgroundSource::Solid(b)) => assert_eq!(a, b),
            _ => panic!("Clone failed or variant mismatch"),
        }
    }

    // ThemeSettings tests
    #[test]
    fn test_theme_settings_default() {
        let theme = ThemeSettings::default();
        assert!(matches!(theme.background, BackgroundSource::Solid(_)));
        assert_eq!(theme.blur_radius, 8);
        assert_eq!(theme.opacity, 0.90);
        assert_eq!(theme.font_size, 24);
    }

    #[test]
    fn test_theme_settings_clone() {
        let theme = ThemeSettings::default();
        let cloned = theme.clone();
        assert_eq!(theme.blur_radius, cloned.blur_radius);
        assert_eq!(theme.opacity, cloned.opacity);
        assert_eq!(theme.font_size, cloned.font_size);
    }

    #[test]
    fn test_theme_settings_custom() {
        let theme = ThemeSettings {
            background: BackgroundSource::ImagePath("/test.jpg".to_string()),
            text_color: HexColor::new("#FF0000"),
            blur_radius: 10,
            opacity: 0.8,
            font_size: 24,
            font_family: FontFamily("Helvetica".to_string()),
        };

        assert!(matches!(theme.background, BackgroundSource::ImagePath(_)));
        assert_eq!(theme.blur_radius, 10);
        assert_eq!(theme.opacity, 0.8);
        assert_eq!(theme.font_size, 24);
    }

    // FontFamily tests
    #[test]
    fn test_font_family_creation() {
        let font = FontFamily("Times New Roman".to_string());
        let serialized = serde_json::to_string(&font).unwrap();
        let _deserialized: FontFamily = serde_json::from_str(&serialized).unwrap();
        assert!(serialized.contains("Times New Roman"));
    }

    // Serialization tests
    #[test]
    fn test_hex_color_serialization() {
        let color = HexColor::new("#ABCDEF");
        let json = serde_json::to_string(&color).unwrap();
        let deserialized: HexColor = serde_json::from_str(&json).unwrap();
        assert_eq!(color.to_string(), deserialized.to_string());
    }

    #[test]
    fn test_background_source_serialization() {
        let bg = BackgroundSource::Solid(HexColor::new("#123456"));
        let json = serde_json::to_string(&bg).unwrap();
        let deserialized: BackgroundSource = serde_json::from_str(&json).unwrap();
        match deserialized {
            BackgroundSource::Solid(color) => assert_eq!(color.to_string(), "#123456"),
            _ => panic!("Deserialization failed"),
        }
    }

    #[test]
    fn test_theme_settings_serialization() {
        let theme = ThemeSettings::default();
        let json = serde_json::to_string(&theme).unwrap();
        let deserialized: ThemeSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(theme.blur_radius, deserialized.blur_radius);
        assert_eq!(theme.opacity, deserialized.opacity);
    }

    // Boundary tests
    #[test]
    fn test_theme_settings_opacity_bounds() {
        let mut theme = ThemeSettings {
            opacity: 0.0,
            ..Default::default()
        };
        assert_eq!(theme.opacity, 0.0);

        theme.opacity = 1.0;
        assert_eq!(theme.opacity, 1.0);
    }

    #[test]
    fn test_theme_settings_blur_radius_max() {
        let theme = ThemeSettings {
            blur_radius: u8::MAX,
            ..Default::default()
        };
        assert_eq!(theme.blur_radius, 255);
    }

    #[test]
    fn test_theme_settings_font_size_range() {
        let mut theme = ThemeSettings {
            font_size: 8,
            ..Default::default()
        };
        assert_eq!(theme.font_size, 8);

        theme.font_size = 72;
        assert_eq!(theme.font_size, 72);
    }
}
