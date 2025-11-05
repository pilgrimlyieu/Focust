use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Type of audio source currently active
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename_all = "camelCase")]
#[derive(Default)]
pub enum AudioSourceType {
    #[default]
    None,
    Builtin,
    FilePath,
}

/// Audio source configuration that preserves all type values
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct AudioSource {
    /// Currently active audio source type
    pub current: AudioSourceType,
    /// Builtin audio name (used when current == Builtin)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builtin_name: Option<String>,
    /// File path (used when current == `FilePath`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            current: AudioSourceType::None,
            builtin_name: None,
            file_path: None,
        }
    }
}

impl AudioSource {
    /// Create a new None audio source
    #[must_use]
    pub fn new_none() -> Self {
        Self {
            current: AudioSourceType::None,
            builtin_name: None,
            file_path: None,
        }
    }

    /// Create a new builtin audio source
    #[must_use]
    pub fn new_builtin(name: String) -> Self {
        Self {
            current: AudioSourceType::Builtin,
            builtin_name: Some(name),
            file_path: None,
        }
    }

    /// Create a new file path audio source
    #[must_use]
    pub fn new_file_path(path: String) -> Self {
        Self {
            current: AudioSourceType::FilePath,
            builtin_name: None,
            file_path: Some(path),
        }
    }

    /// Get the builtin name
    #[must_use]
    pub fn get_builtin_name(&self) -> Option<&str> {
        self.builtin_name.as_deref()
    }

    /// Get the file path
    #[must_use]
    pub fn get_file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    /// Set the builtin name (doesn't change current type)
    pub fn set_builtin_name(&mut self, name: String) {
        self.builtin_name = Some(name);
    }

    /// Set the file path (doesn't change current type)
    pub fn set_file_path(&mut self, path: String) {
        self.file_path = Some(path);
    }

    /// Switch to None audio source
    pub fn use_none(&mut self) {
        self.current = AudioSourceType::None;
    }

    /// Switch to builtin audio source
    pub fn use_builtin(&mut self) {
        self.current = AudioSourceType::Builtin;
        if self.builtin_name.is_none() {
            self.builtin_name = Some(String::new());
        }
    }

    /// Switch to file path audio source
    pub fn use_file_path(&mut self) {
        self.current = AudioSourceType::FilePath;
        if self.file_path.is_none() {
            self.file_path = Some(String::new());
        }
    }
}

/// Audio settings for break sounds
#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export, rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
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
            source: AudioSource::default(),
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
        match self.source.current {
            AudioSourceType::None => None,
            AudioSourceType::Builtin => {
                self.source.builtin_name.as_ref().map(|name| {
                    // Return resource identifier in format: "sounds/{name}.mp3"
                    format!("sounds/{name}.mp3")
                })
            }
            AudioSourceType::FilePath => self.source.file_path.clone(),
        }
    }

    /// Check if this audio source is a builtin resource
    #[must_use]
    pub fn is_builtin(&self) -> bool {
        self.source.current == AudioSourceType::Builtin
    }

    /// Check if this audio source is a file path
    #[must_use]
    pub fn is_file_path(&self) -> bool {
        self.source.current == AudioSourceType::FilePath
    }

    /// Check if this audio source is None
    #[must_use]
    pub fn is_none(&self) -> bool {
        self.source.current == AudioSourceType::None
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_audio_source_default() {
        let source = AudioSource::default();
        assert_eq!(source.current, AudioSourceType::None);
        assert!(source.builtin_name.is_none());
        assert!(source.file_path.is_none());
    }

    #[test]
    fn test_audio_source_new_none() {
        let source = AudioSource::new_none();
        assert_eq!(source.current, AudioSourceType::None);
        assert!(source.builtin_name.is_none());
        assert!(source.file_path.is_none());
    }

    #[test]
    fn test_audio_source_new_builtin() {
        let source = AudioSource::new_builtin("bell".to_string());
        assert_eq!(source.current, AudioSourceType::Builtin);
        assert_eq!(source.builtin_name, Some("bell".to_string()));
        assert!(source.file_path.is_none());
    }

    #[test]
    fn test_audio_source_new_file_path() {
        let source = AudioSource::new_file_path("/path/to/audio.mp3".to_string());
        assert_eq!(source.current, AudioSourceType::FilePath);
        assert!(source.builtin_name.is_none());
        assert_eq!(source.file_path, Some("/path/to/audio.mp3".to_string()));
    }

    #[test]
    fn test_audio_source_getters() {
        let mut source = AudioSource::new_builtin("bell".to_string());
        assert_eq!(source.get_builtin_name(), Some("bell"));
        assert_eq!(source.get_file_path(), None);

        source.set_file_path("/test.mp3".to_string());
        assert_eq!(source.get_file_path(), Some("/test.mp3"));
    }

    #[test]
    fn test_audio_source_switch_preserves_values() {
        let mut source = AudioSource::new_builtin("bell".to_string());

        // Switch to file path
        source.use_file_path();
        assert_eq!(source.current, AudioSourceType::FilePath);
        // Builtin name should be preserved
        assert_eq!(source.get_builtin_name(), Some("bell"));

        // Set file path
        source.set_file_path("/custom.mp3".to_string());
        assert_eq!(source.get_file_path(), Some("/custom.mp3"));

        // Switch back to builtin
        source.use_builtin();
        assert_eq!(source.current, AudioSourceType::Builtin);
        // File path should be preserved
        assert_eq!(source.get_file_path(), Some("/custom.mp3"));
        // Builtin name should still be there
        assert_eq!(source.get_builtin_name(), Some("bell"));
    }

    #[test]
    fn test_audio_source_serialize_new_format() {
        let source = AudioSource {
            current: AudioSourceType::Builtin,
            builtin_name: Some("bell".to_string()),
            file_path: Some("/preserved.mp3".to_string()),
        };

        let json = serde_json::to_string(&source).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["current"], "builtin");
        assert_eq!(parsed["builtinName"], "bell");
        assert_eq!(parsed["filePath"], "/preserved.mp3");
    }

    #[test]
    fn test_audio_source_deserialize_new_format() {
        let json = r#"{"current":"builtin","builtinName":"bell","filePath":"/preserved.mp3"}"#;
        let source: AudioSource = serde_json::from_str(json).unwrap();

        assert_eq!(source.current, AudioSourceType::Builtin);
        assert_eq!(source.get_builtin_name(), Some("bell"));
        assert_eq!(source.get_file_path(), Some("/preserved.mp3"));
    }

    #[test]
    fn test_audio_settings_default() {
        let settings = AudioSettings::default();
        assert_eq!(settings.source.current, AudioSourceType::None);
        assert_eq!(settings.volume, 0.6);
    }

    #[test]
    fn test_audio_settings_get_path_none() {
        let settings = AudioSettings {
            source: AudioSource::new_none(),
            volume: 0.6,
        };
        assert!(settings.get_path().is_none());
    }

    #[test]
    fn test_audio_settings_get_path_builtin() {
        let settings = AudioSettings {
            source: AudioSource::new_builtin("bell".to_string()),
            volume: 0.6,
        };
        assert_eq!(settings.get_path(), Some("sounds/bell.mp3".to_string()));
    }

    #[test]
    fn test_audio_settings_get_path_file_path() {
        let settings = AudioSettings {
            source: AudioSource::new_file_path("/custom/audio.mp3".to_string()),
            volume: 0.6,
        };
        assert_eq!(settings.get_path(), Some("/custom/audio.mp3".to_string()));
    }

    #[test]
    fn test_audio_settings_is_builtin() {
        let mut settings = AudioSettings {
            source: AudioSource::new_builtin("bell".to_string()),
            volume: 0.6,
        };
        assert!(settings.is_builtin());

        settings.source.use_none();
        assert!(!settings.is_builtin());

        settings.source.use_file_path();
        assert!(!settings.is_builtin());
    }

    #[test]
    fn test_audio_source_persists_all_values() {
        // Create source with builtin
        let mut source = AudioSource::new_builtin("bell".to_string());
        source.set_file_path("/custom.mp3".to_string());

        // Serialize to JSON - all fields should be present
        let json = serde_json::to_string(&source).unwrap();
        println!("Serialized JSON: {json}");

        // Verify all fields are present in JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["current"], "builtin");
        assert_eq!(parsed["builtinName"], "bell");
        assert_eq!(parsed["filePath"], "/custom.mp3");

        // Deserialize back
        let restored: AudioSource = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.current, AudioSourceType::Builtin);
        assert_eq!(restored.builtin_name, Some("bell".to_string()));
        assert_eq!(restored.file_path, Some("/custom.mp3".to_string()));

        // Switch to file path - builtin should still be there
        let mut restored = restored;
        restored.use_file_path();
        let json2 = serde_json::to_string(&restored).unwrap();
        let parsed2: serde_json::Value = serde_json::from_str(&json2).unwrap();
        assert_eq!(parsed2["current"], "filePath");
        assert_eq!(parsed2["builtinName"], "bell"); // Still preserved.
        assert_eq!(parsed2["filePath"], "/custom.mp3");
    }

    #[test]
    fn test_audio_source_toml_skips_none_values() {
        // Create source with only builtin (file_path is None)
        let source = AudioSource::new_builtin("bell".to_string());

        // Serialize to TOML
        let toml = toml::to_string(&source).unwrap();
        println!("Serialized TOML:\n{toml}");

        // TOML should only contain "current" and "builtinName", not filePath
        assert!(toml.contains("current = \"builtin\""));
        assert!(toml.contains("builtinName = \"bell\""));
        assert!(!toml.contains("filePath"));

        // Now set file path and serialize again
        let mut source2 = AudioSource::new_builtin("chime".to_string());
        source2.set_file_path("/music.mp3".to_string());
        source2.use_file_path();

        let toml2 = toml::to_string(&source2).unwrap();
        println!("After switch to filePath:\n{toml2}");

        // Should have both builtinName and filePath
        assert!(toml2.contains("current = \"filePath\""));
        assert!(toml2.contains("builtinName = \"chime\"")); // Preserved.
        assert!(toml2.contains("filePath = \"/music.mp3\""));
    }
}
