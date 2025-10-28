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
    /// Solid color background (hex color code)
    Solid(String),
    /// Image background from a specific file path
    ImagePath(String),
    /// Image background from a folder (randomly selected)
    ImageFolder(String),
}

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
            background: BackgroundSource::Solid("#1f2937".to_string()),
            text_color: HexColor("#f8fafc".to_string()),
            blur_radius: 8,
            opacity: 0.9,
            font_size: 24,
            font_family: FontFamily("Arial".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // HexColor validation tests
    #[test]
    fn test_hex_color_valid() {
        let color = HexColor("#FFFFFF".to_string());
        assert!(color.is_valid());
    }

    #[test]
    fn test_hex_color_valid_lowercase() {
        let color = HexColor("#ffffff".to_string());
        assert!(color.is_valid());
    }

    #[test]
    fn test_hex_color_valid_mixed_case() {
        let color = HexColor("#FfFfFf".to_string());
        assert!(color.is_valid());
    }

    #[test]
    fn test_hex_color_invalid_no_hash() {
        let color = HexColor("FFFFFF".to_string());
        assert!(!color.is_valid());
    }

    #[test]
    fn test_hex_color_invalid_too_short() {
        let color = HexColor("#FFF".to_string());
        assert!(!color.is_valid());
    }

    #[test]
    fn test_hex_color_invalid_too_long() {
        let color = HexColor("#FFFFFF00".to_string());
        assert!(!color.is_valid());
    }

    #[test]
    fn test_hex_color_invalid_non_hex_chars() {
        let color = HexColor("#GGGGGG".to_string());
        assert!(!color.is_valid());
    }

    #[test]
    fn test_hex_color_invalid_special_chars() {
        let color = HexColor("#FFF FFF".to_string());
        assert!(!color.is_valid());
    }

    #[test]
    fn test_hex_color_empty() {
        let color = HexColor("".to_string());
        assert!(!color.is_valid());
    }

    #[test]
    fn test_hex_color_only_hash() {
        let color = HexColor("#".to_string());
        assert!(!color.is_valid());
    }

    // BackgroundSource tests
    #[test]
    fn test_background_source_solid() {
        let bg = BackgroundSource::Solid("#000000".to_string());
        match bg {
            BackgroundSource::Solid(color) => assert_eq!(color, "#000000"),
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
        let bg = BackgroundSource::Solid("#123456".to_string());
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
        assert_eq!(theme.opacity, 0.92);
        assert_eq!(theme.font_size, 24);
    }

    #[test]
    fn test_theme_settings_default_valid_color() {
        let theme = ThemeSettings::default();
        assert!(theme.text_color.is_valid());
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
            text_color: HexColor("#FF0000".to_string()),
            blur_radius: 10,
            opacity: 0.8,
            font_size: 24,
            font_family: FontFamily("Helvetica".to_string()),
        };

        assert!(matches!(theme.background, BackgroundSource::ImagePath(_)));
        assert!(theme.text_color.is_valid());
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
        let color = HexColor("#ABCDEF".to_string());
        let json = serde_json::to_string(&color).unwrap();
        let deserialized: HexColor = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_valid());
    }

    #[test]
    fn test_background_source_serialization() {
        let bg = BackgroundSource::Solid("#123456".to_string());
        let json = serde_json::to_string(&bg).unwrap();
        let deserialized: BackgroundSource = serde_json::from_str(&json).unwrap();
        match deserialized {
            BackgroundSource::Solid(color) => assert_eq!(color, "#123456"),
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
