# Configuration Reference

<div align="center">

**[简体中文](CONFIGURATION.zh-CN.md)** | **[English](CONFIGURATION.md)**

</div>

> [!WARNING]
> Due to implementation limitations, configuration options cannot use TOML standard snake_case and are all in camelCase.

This document provides a comprehensive guide to all configuration options available in Focust. Configuration is stored in TOML format and can be edited either through the settings UI or by manually editing the configuration file.

## Table of Contents

- [Configuration File Location](#configuration-file-location)
- [Configuration Structure](#configuration-structure)
- [General Settings](#general-settings)
- [Break Schedule Settings](#break-schedule-settings)
- [Attention Settings](#attention-settings)
- [Theme Settings](#theme-settings)
- [Audio Settings](#audio-settings)
- [Suggestion Settings](#suggestion-settings)
- [Examples](#examples)
- [Tips and Best Practices](#tips-and-best-practices)

---

## Configuration File Location

Focust stores its configuration in a platform-specific location:

- **Windows**: `%APPDATA%\com.fesmoph.focust\config.toml`
- **macOS**: `~/Library/Application Support/com.fesmoph.focust/config.toml`
- **Linux**: `~/.config/com.fesmoph.focust/config.toml`

You can quickly open the configuration directory from the settings UI:
1. Go to **Advanced** tab
2. Click **Open configuration directory**

## Configuration Structure

The configuration file is divided into several main sections:

```toml
# General application settings
checkForUpdates = true
autostart = false
monitorDnd = true
# ...

# Break schedules (array of tables)
[[schedules]]
name = "Work Hours"
enabled = true
# ...

# Timed reminders (array of tables)
[[attentions]]
name = "Water Reminder"
enabled = true
# ...
```

---

## General Settings

### `checkForUpdates`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Automatically check for application updates on startup
- **NOTE**: Unimplemented yet

### `autostart`
- **Type**: Boolean
- **Default**: `false`
- **Description**: Automatically start the application when the system boots up

### `monitorDnd`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Pause all breaks when system Do Not Disturb mode is detected

### `inactiveS`
- **Type**: Integer (seconds)
- **Default**: `300` (5 minutes)
- **Description**: Duration of inactivity before the scheduler automatically pauses. The scheduler resumes when activity is detected.

### `allScreens`
- **Type**: Boolean
- **Default**: `false`
- **Description**: Show break windows on all monitors in a multi-monitor setup. When disabled, breaks only appear on the primary monitor.

### `language`
- **Type**: String
- **Default**: Auto-detected from system locale
- **Options**: `"en-US"`, `"zh-CN"`
- **Description**: Interface language. Automatically detected on first run but can be changed manually.

### `themeMode`
- **Type**: String
- **Default**: `"system"`
- **Options**: `"light"`, `"dark"`, `"system"`
- **Description**: Color theme for the settings UI. `"system"` follows your operating system's theme preference.

### `postponeShortcut`
- **Type**: String
- **Default**: `""` (empty)
- **Format**: Key combination like `"Ctrl+Shift+P"` or `"Alt+P"`
- **Description**: Global hotkey to postpone the next break. Leave empty to disable. Examples:
  - `"Ctrl+Shift+P"`
  - `"Alt+B"`

### `windowSize`
- **Type**: Float
- **Default**: `0.8` (80% of screen size)
- **Range**: `0.5` to `1.0`
- **Description**: Size of the break window as a fraction of the screen.
  - `0.5` = 50% (half screen)
  - `0.8` = 80% (default)
  - `1.0` = 100% (fullscreen)

**Example:**
```toml
checkForUpdates = true
autostart = false
monitorDnd = true
inactiveS = 300
allScreens = false
language = "en-US"
themeMode = "system"
postponeShortcut = "Ctrl+Shift+P"
windowSize = 0.8
```

---

## Break Schedule Settings

Break Schedules define when and how often breaks occur. You can have multiple schedules with different time ranges and active days.

### Basic Break Schedule Fields

#### `id`
- **Type**: Integer
- **Description**: Unique identifier for the schedule, used internally by the application. Should not be modified manually!
- **Note**: Omitted in this documentation for simplicity.
 
#### `name`
- **Type**: String
- **Default**: `"Default Schedule"`
- **Description**: Human-readable name for the schedule

#### `enabled`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Whether this schedule is active

#### `notificationBeforeS`
- **Type**: Integer (seconds)
- **Default**: `10`
- **Description**: Send a system notification X seconds before a break starts. Set to `0` to disable notifications.

### Time Range

#### `timeRange`
- **Type**: Table
- **Description**: Defines when this schedule is active during the day. Set both start and end times as "00:00" to represent a full-day schedule.

```toml
[schedules.timeRange]
start = "09:00"  # 24-hour format
end = "17:00"    # 24-hour format

# For full-day schedule
[schedules.timeRange]
start = "00:00"
end = "00:00"

# The app may save as hh:mm:ss internally
[schedules.timeRange]
start = "00:00:00"
end = "00:00:00"
```

#### `daysOfWeek`
- **Type**: Array of strings
- **Default**: `["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]`
- **Options**: `"Mon"`, `"Tue"`, `"Wed"`, `"Thu"`, `"Fri"`, `"Sat"`, `"Sun"`
- **Description**: Days when this schedule is active

### Mini Breaks

Mini breaks are short reminders (typically lasting 20 seconds) that occur frequently.

#### `miniBreaks.enabled`
- **Type**: Boolean
- **Default**: `true`

#### `miniBreaks.durationS`
- **Type**: Integer (seconds)
- **Default**: `20`
- **Description**: How long the mini break lasts

#### `miniBreaks.postponedS`
- **Type**: Integer (seconds)
- **Default**: `300` (5 minutes)
- **Description**: How long to postpone the break when using the postpone function

#### `miniBreaks.strictMode`
- **Type**: Boolean
- **Default**: `false`
- **Description**: When enabled, breaks cannot be skipped or postponed

#### `miniBreaks.intervalS`
- **Type**: Integer (seconds)
- **Default**: `1200` (20 minutes)
- **Description**: Time between mini breaks

#### Mini Break Theme

See [Theme Settings](#theme-settings) section for theme configuration details.

```toml
[schedules.miniBreaks.theme]
background = { solid = "#1f2937" }
textColor = "#f8fafc"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"
```

#### Mini Break Audio

See [Audio Settings](#audio-settings) section for audio configuration details.

```toml
[schedules.miniBreaks.audio]
current = "builtin"
name = "gentle-bell"
volume = 0.7
```

#### Mini Break Suggestions

See [Suggestion Settings](#suggestion-settings) section for suggestion configuration details.

```toml
[schedules.miniBreaks.suggestions]
show = true
```

### Long Breaks

Long breaks are extended rest periods (typically 5 minutes) that occur less frequently.

#### `longBreaks.enabled`
- **Type**: Boolean
- **Default**: `true`

#### `longBreaks.durationS`
- **Type**: Integer (seconds)
- **Default**: `300` (5 minutes)

#### `longBreaks.postponedS`
- **Type**: Integer (seconds)
- **Default**: `300` (5 minutes)

#### `longBreaks.strictMode`
- **Type**: Boolean
- **Default**: `false`

#### `longBreaks.afterMiniBreaks`
- **Type**: Integer
- **Default**: `4`
- **Description**: Trigger a long break after this many mini breaks. For example, with default settings:
  - Mini break at 20 min, 40 min, 60 min, 80 min
  - Long break at 100 min (after 4 mini breaks)
  - Cycle repeats

Long breaks also support theme, audio, and suggestion settings (same format as mini breaks).

**Example Schedule:**
```toml
[[schedules]]
name = "Work Hours"
enabled = true
notificationBeforeS = 5
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri"]

[schedules.timeRange]
start = "09:00"
end = "17:00"

[schedules.miniBreaks]
enabled = true
durationS = 20
postponedS = 300
strictMode = false
intervalS = 1200

[schedules.miniBreaks.theme]
background = { solid = "#1e293b" }
textColor = "#f1f5f9"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"

[schedules.miniBreaks.audio]
current = "builtin"
name = "gentle-bell"
volume = 0.7

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
enabled = true
durationS = 300
postponedS = 600
strictMode = false
afterMiniBreaks = 4

[schedules.longBreaks.theme]
background = { imagePath = "/path/to/background.jpg" }
textColor = "#ffffff"
blurRadius = 10
opacity = 0.85
fontSize = 28
fontFamily = "Helvetica"

[schedules.longBreaks.audio]
current = "filePath"
path = "/path/to/sound.mp3"
volume = 0.8

[schedules.longBreaks.suggestions]
show = true
```

---

## Attention Settings

Attention reminders are time-based notifications that work like alarm clocks. Unlike breaks, they don't interrupt your work — just show a brief message at specific times, which can be dismissed immediately.

### Basic Attention Fields

#### `name`
- **Type**: String
- **Required**: Yes
- **Description**: Name of the reminder (e.g., "Water Reminder", "Stand Up Alert")

#### `enabled`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Whether this reminder is active

#### `times`
- **Type**: Array of strings (24-hour time format)
- **Required**: Yes
- **Description**: List of times when the reminder should trigger

#### `daysOfWeek`
- **Type**: Array of strings
- **Default**: `["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]`
- **Options**: `"Mon"`, `"Tue"`, `"Wed"`, `"Thu"`, `"Fri"`, `"Sat"`, `"Sun"`

#### `title`
- **Type**: String
- **Default**: `"Attention"`
- **Description**: Title shown in the popup window

#### `message`
- **Type**: String
- **Required**: Yes
- **Description**: The message content to display

#### `durationS`
- **Type**: Integer (seconds)
- **Default**: `10`
- **Description**: How long to show the reminder popup

Attention reminders also support theme, audio, and suggestion settings.

**Example Attentions:**
```toml
# Water reminder
[[attentions]]
name = "Hydrate"
enabled = true
times = ["10:00", "14:00", "16:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri"]
title = "Time to Drink Water"
message = "Stay hydrated! Drink a glass of water."
durationS = 10

[attentions.theme]
background = { solid = "#0ea5e9" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 20
fontFamily = "Arial"

[attentions.audio]
current = "builtin"
name = "notification"
volume = 0.6

# Stand up reminder
[[attentions]]
name = "Stand Up"
enabled = true
times = ["11:00", "15:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
title = "Stand Up and Move"
message = "Take a moment to stand up and stretch your legs!"
durationS = 15

[attentions.theme]
background = { solid = "#10b981" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 20
fontFamily = "Arial"

[attentions.audio]
current = "none"
```

---

## Theme Settings

Theme settings control the appearance of break windows.

### `background`
- **Type**: Object
- **Description**: Background source for the break window

**Structure:**

```toml
[theme.background]
current = "solid"                # Currently active background type
solid = "#1f2937"                # Preserved even when switched to image
imagePath = "/path/to/image.jpg" # Preserved even when switched to folder
imageFolder = "/path/to/images/" # Preserved even when switched to solid
```

**Background Type Options:**

1. **Solid Color** (`current = "solid"`):
   ```toml
   [theme.background]
   current = "solid"
   solid = "#1f2937"
   ```

2. **Single Image** (`current = "imagePath"`):
   ```toml
   [theme.background]
   current = "imagePath"
   imagePath = "/path/to/image.jpg"
   ```
   - Supported Formats: JPG, PNG, WebP, etc.

3. **Random Image from Folder** (`current = "imageFolder"`):
   ```toml
   [theme.background]
   current = "imageFolder"
   imageFolder = "/path/to/images/"
   ```

### `textColor`
- **Type**: String (hex color)
- **Default**: `"#f8fafc"`
- **Format**: `#RRGGBB`
- **Description**: Color of all text in the break window

### `blurRadius`
- **Type**: Integer (0-255)
- **Default**: `8`
- **Description**: Gaussian blur radius applied to the background image (in pixels)

### `opacity`
- **Type**: Float (0.0-1.0)
- **Default**: `0.9`
- **Description**: Opacity of the break window overlay
  - `0.0` = Completely transparent (content still visible)
  - `1.0` = Completely opaque

### `fontSize`
- **Type**: Integer (pixels)
- **Default**: `24`
- **Description**: Base font size for text in the break window

### `fontFamily`
- **Type**: String
- **Default**: `"Arial"`
- **Description**: Font family name. Use fonts installed on the system.

**Theme Examples:**

```toml
# Minimal dark theme
[theme.background]
current = "solid"
solid = "#0f172a"

[theme]
textColor = "#e2e8f0"
blurRadius = 0
opacity = 1.0
fontSize = 20
fontFamily = "Segoe UI"

# Nature theme with image
[theme.background]
current = "imagePath"
imagePath = "/path/to/forest.jpg"

[theme]
textColor = "#ffffff"
blurRadius = 12
opacity = 0.75
fontSize = 28
fontFamily = "Georgia"

# Vibrant theme with preserved settings
[theme.background]
current = "solid"
solid = "#7c3aed"
imagePath = "/path/to/sunset.jpg" # Preserved
imageFolder = "C:\\Wallpapers"    # Preserved

[theme]
textColor = "#fef3c7"
blurRadius = 0
opacity = 0.95
fontSize = 26
fontFamily = "Tahoma"
```

---

## Audio Settings

Audio settings control sound playback during breaks.

**Structure**:

```toml
[audio]
current = "builtin"              # Currently active audio type
builtinName = "gentle-bell"      # Preserved even when switched to filePath
filePath = "/path/to/custom.mp3" # Preserved even when switched to builtin
volume = 0.6
```

### `current`
- **Type**: String enum
- **Default**: `"none"`
- **Options**: `"none"`, `"builtin"`, `"filePath"`
- **Description**: Currently active audio type

### `builtinName`
- **Type**: String (optional)
- **Description**: Name of the built-in sound effect
- **Available sounds**:
  - `"gentle-bell"` - Soft bell chime
  - `"soft-gong"` - Gentle gong sound
  - `"notification"` - Simple notification beep
  - `"bright-notification"` - Upbeat notification

### `filePath`
- **Type**: String (optional)
- **Description**: Absolute path to a custom audio file
- **Supported formats**: MP3, WAV, OGG, FLAC

### `volume`
- **Type**: Float (0.0-1.0)
- **Default**: `0.6`
- **Description**: Playback volume
  - `0.0` = Muted
  - `1.0` = Maximum volume

**Audio Examples:**

```toml
# No sound
[audio]
current = "none"
volume = 0.6

# Built-in sound
[audio]
current = "builtin"
builtinName = "gentle-bell"
volume = 0.7

# Custom audio file
[audio]
current = "filePath"
filePath = "C:\\Users\\YourName\\Music\\zen-bell.mp3"
volume = 0.8
```

---

## Suggestion Settings

Suggestion settings control whether motivational messages or tips are shown during breaks.

### `show`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Show a random suggestion during each break

**Suggestion Configuration:**

Suggestions themselves are stored in separate TOML files located in the same directory as the main config file:
- `suggestions.toml`

**Suggestion File Format:**

```toml
[byLanguage.en-US]
suggestions = [
    "Look away from your screen and focus on a distant object.",
    "Roll your shoulders back and take a deep breath.",
    "Drink a glass of water.",
    ...
]

[byLanguage.zh-CN]
suggestions = [
    "将目光从屏幕移开，专注于远处的物体。",
    "向后转动肩膀，深呼吸。",
    "喝一杯水。",
    ...
]
```

You can add your own suggestions under the appropriate language section. The app randomly selects one suggestion to display during each break.

---

## Examples

### Example 1: Minimal Configuration

```toml
# General settings
checkForUpdates = true
autostart = false
monitorDnd = true
inactiveS = 300
allScreens = false
language = "zh-CN"
themeMode = "system"
postponeShortcut = ""
windowSize = 0.8
attentions = []

[[schedules]]
name = "Default Schedule"
enabled = true
daysOfWeek = [
    "Mon",
    "Tue",
    "Wed",
    "Thu",
    "Fri",
    "Sat",
    "Sun",
]
notificationBeforeS = 10

[schedules.timeRange]
start = "00:00:00"
end = "00:00:00"

[schedules.miniBreaks]
id = 0
enabled = true
durationS = 20
postponedS = 300
strictMode = false
intervalS = 1200

[schedules.miniBreaks.theme]
textColor = "#f8fafc"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"

[schedules.miniBreaks.theme.background]
solid = "#1f2937"

[schedules.miniBreaks.audio]
current = "none"
volume = 0.6

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
id = 1
enabled = true
durationS = 300
postponedS = 300
strictMode = false
afterMiniBreaks = 4

[schedules.longBreaks.theme]
textColor = "#f8fafc"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"

[schedules.longBreaks.theme.background]
solid = "#1f2937"

[schedules.longBreaks.audio]
current = "none"
volume = 0.6

[schedules.longBreaks.suggestions]
show = true
```

### Example 2: Comprehensive Work Setup

```toml
checkForUpdates = true
monitorDnd = true
inactiveS = 300
allScreens = true
language = "en-US"
themeMode = "system"
postponeShortcut = "Ctrl+Shift+B"
windowSize = 0.85

# Work hours schedule
[[schedules]]
name = "Work Hours"
enabled = true
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri"]
notificationBeforeS = 10

[schedules.timeRange]
start = "09:00"
end = "17:30"

[schedules.miniBreaks]
enabled = true
durationS = 20
postponedS = 300
strictMode = false
intervalS = 1200  # 20 min

[schedules.miniBreaks.theme]
background = { imagePath = "/home/user/wallpapers/calm-lake.jpg" }
textColor = "#e0f2fe"
blurRadius = 10
opacity = 0.85
fontSize = 24
fontFamily = "Helvetica"

[schedules.miniBreaks.audio]
current = "builtin"
name = "gentle-bell"
volume = 0.6

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
enabled = true
durationS = 600  # 10 min
postponedS = 600
strictMode = true
afterMiniBreaks = 3

[schedules.longBreaks.theme]
background = { imagePath = "/home/user/wallpapers/mountain.jpg" }
textColor = "#ffffff"
blurRadius = 15
opacity = 0.8
fontSize = 28
fontFamily = "Georgia"

[schedules.longBreaks.audio]
current = "filePath"
path = "/home/user/sounds/meditation-bell.mp3"
volume = 0.7

[schedules.longBreaks.suggestions]
show = true

# Evening schedule (shorter breaks, less strict)
[[schedules]]
name = "Evening"
enabled = true
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
notificationBeforeS = 5

[schedules.timeRange]
start = "18:00"
end = "22:00"

[schedules.miniBreaks]
enabled = true
durationS = 15
postponedS = 600
strictMode = false
intervalS = 1800  # 30 min

[schedules.miniBreaks.theme]
background = { Solid = "#1e3a8a" }
textColor = "#dbeafe"
blurRadius = 5
opacity = 0.9
fontSize = 22
fontFamily = "Arial"

[schedules.miniBreaks.audio]
current = "builtin"
name = "soft-gong"
volume = 0.5

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
enabled = false

# Hydration reminder
[[attentions]]
name = "Drink Water"
enabled = true
times = ["10:00", "14:00", "16:00", "20:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
title = "Hydration Time"
message = "Remember to drink water! Stay hydrated throughout the day."
durationS = 10

[attentions.theme]
background = { solid = "#0ea5e9" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 20
fontFamily = "Arial"

[attentions.audio]
current = "builtin"
name = "notification"
volume = 0.5

# Eye exercise reminder
[[attentions]]
name = "Eye Exercise"
enabled = true
times = ["11:00", "15:00", "19:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri"]
title = "Eye Care"
message = "Look at something 20 feet away for 20 seconds (20-20-20 rule)"
durationS = 25

[attentions.theme]
background = { solid = "#10b981" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 22
fontFamily = "Arial"

[attentions.audio]
current = "builtin"
name = "bright-notification"
volume = 0.6
```

---

## Tips and Best Practices

1. **Start Simple**: Begin with the default configuration and gradually customize
2. **Test Settings**: Use the preview/test buttons in the settings UI before saving
3. **Backup Config**: Keep a backup of your configuration file, especially before major changes
4. **Use Strict Mode Wisely**: Only enable strict mode for breaks you absolutely need to take
5. **Image Paths**: Use absolute paths for images to avoid issues
6. **Audio Volume**: Start with lower volumes (0.5-0.7) and adjust as needed
7. **Multiple Schedules**: Use different schedules for different times of day or days of the week
8. **Attention Timing**: Space out attention reminders to avoid notification fatigue

---

For more information, see:
- [README.md](../README.md) - Project overview and features
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Development guide
