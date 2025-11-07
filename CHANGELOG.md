# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## 0.2.2 (2025.11.7)

### 🐛 Bug Fixes

- Fix critical bug that when prompt window is set to fullscreen, DND mode will be enabled (this's Focus Assist default behavior on Windows), which causes scheduler pause immediately.

### 🚀 Improvements

- DND feature will not cause panic on Windows platform anymore. (Firstly mentioned in 0.2.1)

## 0.2.1 (2025.11.6)

### 🚀 Improvements

- Update 8 languages support: Japanese, German, French, Spanish, Russian, Portuguese, Italian, Korean.
- Disable noisy upstream logs.
- ~~DND feature will not cause panic on Windows platform anymore.~~ (Fixed in 0.2.2)

## 0.2.0 (2025.11.6)

### 🎉 Features

- Add DND(Do Not Disturb) mode detection on Windows, Linux, and macOS to automatically pause break reminders when Focus Assist or equivalent mode is active.
  - Windows uses WNF API, Linux uses D-Bus, macOS uses polling. Welcomes users to enable it via `monitorDnd` setting and provide feedback.
  - Windows platform has been tested, while Linux and macOS has not.
- Add App Exclusion feature to whitelist applications that temporarily disable break reminders when they are in the foreground or based on more complex rules.
  - Users can configure excluded apps via `excludedApps` setting in the config file.

### 🚀 Improvements

- Pause and Resume action now work for Attention Timer too.
- Use [user-Idle2](https://crates.io/crates/user-idle2) instead of [user-Idle](https://crates.io/crates/user-idle) for Linux Wayland idle detection.
- Hide debug section in advanced settings panel.

### 📝 Documentation

- Update related documentation for DND monitoring and App Exclusion features.

## 0.1.4 (2025.11.5)

### 🚀 Improvements

- Supports retaining old values when switching background and audio settings

### 📝 Documentation

- Updates related infomation in CONFIGURATION.md

### ⚠️ Breaking Changes

- Structure of `background` and `audio` settings in configuration file has changed. If you encountered compatibility issues (impossible I think since no one use now), please update your config file manually according to the following example:

For `background` setting:

```toml
[schedules.miniBreaks.theme.background]
solid = "#cedae9"

# or
[schedules.miniBreaks.theme.background]
imagePath = "/path/to/your/image.png"

# or
[schedules.miniBreaks.theme.background]
imageFolder = "/path/to/your/folder"
```

should be changed to

```toml
[schedules.miniBreaks.theme.background]
current = "solid" # options: "solid", "imagePath", "imageFolder"
solid = "#cedae9"
imagePath = "/path/to/your/image.png"
imageFolder = "/path/to/your/folder"
```

For `audio` setting:

```toml
[schedules.longBreaks.audio]
source = "builtin"
name = "gentle-bell"
volume = 0.6

# or
[schedules.longBreaks.audio]
source = "filePath"
path = "/path/to/your/audio.mp3"
volume = 0.6
```

should be changed to

```toml
# NOTE: `source` is renamed to `current`, and `name`/`path` are renamed to `builtinName`/`filePath`
[schedules.longBreaks.audio]
current = "builtin" # options: "none", "builtin", "filePath"
builtinName = "gentle-bell"
filePath = "/path/to/your/audio.mp3"
volume = 0.6
```

## 0.1.3 (2025.11.5) 🎉 FIRST RELEASE

Initial release.