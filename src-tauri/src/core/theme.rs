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

/// Type of background source currently active
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
#[derive(Default)]
pub enum BackgroundType {
    #[default]
    Solid,
    ImagePath,
    ImageFolder,
}

/// Background source configuration that preserves all type values
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct BackgroundSource {
    /// Currently active background type
    pub current: BackgroundType,
    /// Solid color value (used when current == Solid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solid: Option<HexColor>,
    /// Image file path (used when current == `ImagePath`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
    /// Image folder path (used when current == `ImageFolder`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_folder: Option<String>,
}

impl Default for BackgroundSource {
    fn default() -> Self {
        Self {
            current: BackgroundType::Solid,
            solid: Some(HexColor::default()),
            image_path: None,
            image_folder: None,
        }
    }
}

impl BackgroundSource {
    /// Create a new solid color background
    #[must_use]
    pub fn new_solid(color: HexColor) -> Self {
        Self {
            current: BackgroundType::Solid,
            solid: Some(color),
            ..Default::default()
        }
    }

    /// Create a new image path background
    #[must_use]
    pub fn new_image_path(path: String) -> Self {
        Self {
            current: BackgroundType::ImagePath,
            image_path: Some(path),
            ..Default::default()
        }
    }

    /// Create a new image folder background
    #[must_use]
    pub fn new_image_folder(folder: String) -> Self {
        Self {
            current: BackgroundType::ImageFolder,
            image_folder: Some(folder),
            ..Default::default()
        }
    }

    /// Get the current solid color value
    #[must_use]
    pub fn get_solid(&self) -> Option<&HexColor> {
        self.solid.as_ref()
    }

    /// Get the current image path value
    #[must_use]
    pub fn get_image_path(&self) -> Option<&str> {
        self.image_path.as_deref()
    }

    /// Get the current image folder value
    #[must_use]
    pub fn get_image_folder(&self) -> Option<&str> {
        self.image_folder.as_deref()
    }

    /// Set the solid color value (doesn't change current type)
    pub fn set_solid(&mut self, color: HexColor) {
        self.solid = Some(color);
    }

    /// Set the image path value (doesn't change current type)
    pub fn set_image_path(&mut self, path: String) {
        self.image_path = Some(path);
    }

    /// Set the image folder value (doesn't change current type)
    pub fn set_image_folder(&mut self, folder: String) {
        self.image_folder = Some(folder);
    }

    /// Switch to solid color background
    pub fn use_solid(&mut self) {
        self.current = BackgroundType::Solid;
        if self.solid.is_none() {
            self.solid = Some(HexColor::default());
        }
    }

    /// Switch to image path background
    pub fn use_image_path(&mut self) {
        self.current = BackgroundType::ImagePath;
        if self.image_path.is_none() {
            self.image_path = Some(String::new());
        }
    }

    /// Switch to image folder background
    pub fn use_image_folder(&mut self) {
        self.current = BackgroundType::ImageFolder;
        if self.image_folder.is_none() {
            self.image_folder = Some(String::new());
        }
    }
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
            background: BackgroundSource::new_solid(HexColor::new("#1f2937")),
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
        let bg = BackgroundSource::new_solid(HexColor::new("#000000"));
        assert_eq!(bg.current, BackgroundType::Solid);
        assert_eq!(bg.get_solid().unwrap().to_string(), "#000000");
    }

    #[test]
    fn test_background_source_image_path() {
        let bg = BackgroundSource::new_image_path("/path/to/image.jpg".to_string());
        assert_eq!(bg.current, BackgroundType::ImagePath);
        assert_eq!(bg.get_image_path().unwrap(), "/path/to/image.jpg");
    }

    #[test]
    fn test_background_source_image_folder() {
        let bg = BackgroundSource::new_image_folder("/path/to/folder".to_string());
        assert_eq!(bg.current, BackgroundType::ImageFolder);
        assert_eq!(bg.get_image_folder().unwrap(), "/path/to/folder");
    }

    #[test]
    fn test_background_source_switch_preserves_values() {
        let mut bg = BackgroundSource::new_solid(HexColor::new("#ABCDEF"));

        // Set other values
        bg.set_image_path("/test.jpg".to_string());
        bg.set_image_folder("/test/folder".to_string());

        // Switch to image path
        bg.use_image_path();
        assert_eq!(bg.current, BackgroundType::ImagePath);
        assert_eq!(bg.get_image_path().unwrap(), "/test.jpg");

        // Original solid color should still be preserved
        assert_eq!(bg.get_solid().unwrap().to_string(), "#ABCDEF");

        // Switch back to solid
        bg.use_solid();
        assert_eq!(bg.current, BackgroundType::Solid);
        assert_eq!(bg.get_solid().unwrap().to_string(), "#ABCDEF");

        // Image path should still be preserved
        assert_eq!(bg.get_image_path().unwrap(), "/test.jpg");
    }

    #[test]
    fn test_background_source_clone() {
        let mut bg = BackgroundSource::new_solid(HexColor::new("#123456"));
        bg.set_image_path("/test.jpg".to_string());

        let cloned = bg.clone();
        assert_eq!(cloned.current, bg.current);
        assert_eq!(cloned.get_solid(), bg.get_solid());
        assert_eq!(cloned.get_image_path(), bg.get_image_path());
    }

    #[test]
    fn test_background_source_new_format() {
        // New format with all values preserved
        let new_format = r##"{"current":"solid","solid":"#ABCDEF","imagePath":"/test.jpg","imageFolder":"/folder"}"##;
        let bg: BackgroundSource = serde_json::from_str(new_format).unwrap();

        assert_eq!(bg.current, BackgroundType::Solid);
        assert_eq!(bg.get_solid().unwrap().to_string(), "#ABCDEF");
        assert_eq!(bg.get_image_path().unwrap(), "/test.jpg");
        assert_eq!(bg.get_image_folder().unwrap(), "/folder");
    }

    // ThemeSettings tests
    #[test]
    fn test_theme_settings_default() {
        let theme = ThemeSettings::default();
        assert_eq!(theme.background.current, BackgroundType::Solid);
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
            background: BackgroundSource::new_image_path("/test.jpg".to_string()),
            text_color: HexColor::new("#FF0000"),
            blur_radius: 10,
            opacity: 0.8,
            font_size: 24,
            font_family: FontFamily("Helvetica".to_string()),
        };

        assert_eq!(theme.background.current, BackgroundType::ImagePath);
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
        let mut bg = BackgroundSource::new_solid(HexColor::new("#123456"));
        bg.set_image_path("/preserved.jpg".to_string());

        let json = serde_json::to_string(&bg).unwrap();
        let deserialized: BackgroundSource = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.current, BackgroundType::Solid);
        assert_eq!(deserialized.get_solid().unwrap().to_string(), "#123456");
        assert_eq!(deserialized.get_image_path().unwrap(), "/preserved.jpg");
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
