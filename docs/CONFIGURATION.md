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
- [Application Exclusion Settings](#application-exclusion-settings)
- [Advanced Settings](#advanced-settings)
- [Examples](#examples)
- [Tips and Best Practices](#tips-and-best-practices)

---

## Configuration File Location

Focust stores its configuration in a platform-specific location:

**Production (Release) Builds:**
- **Windows**: `%APPDATA%\com.fesmoph.focust\config.toml`
- **macOS**: `~/Library/Application Support/com.fesmoph.focust/config.toml`
- **Linux**: `~/.config/com.fesmoph.focust/config.toml`

**Development (Debug) Builds:**
- **Windows**: `%APPDATA%\com.fesmoph.focust.dev\config.toml`
- **macOS**: `~/Library/Application Support/com.fesmoph.focust.dev/config.toml`
- **Linux**: `~/.config/com.fesmoph.focust.dev/config.toml`

> **Note:** Development builds use a `.dev` suffix to separate development and production configurations. The `.dev` suffix is appended to the final path component returned by the platform — on Windows and Linux this causes the logs directory to be stored as `.../com.fesmoph.focust/logs.dev`, while on macOS the logs directory becomes `~/Library/Logs/com.fesmoph.focust.dev/`. This allows developers to test changes without affecting their production settings.

You can quickly open the configuration directory from the settings UI:
1. Go to **Advanced Options** tab
2. Click **Open configuration directory**

> **Note:** Focust reads the file once at startup. A missing required field, an invalid enum value, or a table that is not written in full (`theme`, `audio`, `suggestions` and `timeRange` require every required key once the table is present) makes the whole file fail to load: Focust then starts with default settings, and the file is overwritten with those defaults the next time you save from the settings UI. Manual edits made while Focust is running are lost for the same reason. Colour strings and font names are stored as-is and are not validated.

## Configuration Structure

The configuration file is divided into several main sections:

```toml
# General application settings
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

# Application exclusions (array of tables)
[[appExclusions]]
active = true
rule = "pause"
processes = ["chrome.exe", "firefox"]
# ...
```

---

## General Settings

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

### `showTrayIcon`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Display the application icon in the system tray. Requires application restart to take effect.
- **Note**: When hidden, you can reopen settings by launching the app again (uses single instance), or close the app via Task Manager (Windows) / Activity Monitor (macOS) / System Monitor (Linux).

### `language`
- **Type**: String
- **Default**: Auto-detected from system locale
- **Options**: `"de-DE"`, `"en-US"`, `"es-ES"`, `"fr-FR"`, `"it-IT"`, `"ja-JP"`, `"ko-KR"`, `"pt-BR"`, `"ru-RU"`, `"zh-CN"`
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

---

## Break Schedule Settings

Break Schedules define when and how often breaks occur. You can have multiple schedules with different time ranges and active days. Each mini and long break has an internal `id` that the application generates; it can normally be left out and should not be edited manually.

### Basic Break Schedule Fields

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
- **Default**: `{ start = "00:00:00", end = "00:00:00" }`
- **Description**: Defines when this schedule is active during the day. Set both start and end times as "00:00" to represent a full-day schedule. Once the table is present, both `start` and `end` are required.

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

#### `miniBreaks.id`
- **Type**: Integer
- **Required**: No
- **Description**: Internal identifier of this mini break, used to match break events to this entry. Ids must be unique among all `miniBreaks.id` values — with duplicates, the first matching entry is used. The field may be omitted, in which case the application generates a value, but that value is not checked against the ids already in the file, so mixing explicit and omitted ids can produce duplicates.

#### `miniBreaks.enabled`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Whether this mini break is active

#### `miniBreaks.durationS`
- **Type**: Integer (seconds)
- **Default**: `20`
- **Description**: How long the mini break lasts

#### `miniBreaks.postponedS`
- **Type**: Integer (seconds)
- **Default**: `300` (5 minutes)
- **Description**: How long to postpone the break when using the postpone function

#### `miniBreaks.maxPostponeCount`
- **Type**: Integer
- **Default**: `2`
- **Description**: Maximum number of times a mini break can be postponed

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
background = { current = "solid", solid = "#1f2937" }
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
builtinName = "gentle-bell"
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

#### `longBreaks.id`
- **Type**: Integer
- **Required**: No
- **Description**: Internal identifier of this long break, used to match break events to this entry. Ids must be unique among all `longBreaks.id` values — with duplicates, the first matching entry is used. The field may be omitted, in which case the application generates a value, but that value is not checked against the ids already in the file, so mixing explicit and omitted ids can produce duplicates.

#### `longBreaks.enabled`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Whether this long break is active

#### `longBreaks.durationS`
- **Type**: Integer (seconds)
- **Default**: `300` (5 minutes)
- **Description**: How long the long break lasts

#### `longBreaks.postponedS`
- **Type**: Integer (seconds)
- **Default**: `300` (5 minutes)
- **Description**: How long to postpone the break when using the postpone function

#### `longBreaks.maxPostponeCount`
- **Type**: Integer
- **Default**: `2`
- **Description**: Maximum number of times a long break can be postponed

#### `longBreaks.strictMode`
- **Type**: Boolean
- **Default**: `false`
- **Description**: When enabled, breaks cannot be skipped or postponed

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
maxPostponeCount = 2
strictMode = false
intervalS = 1200

[schedules.miniBreaks.theme]
background = { current = "solid", solid = "#1e293b" }
textColor = "#f1f5f9"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"

[schedules.miniBreaks.audio]
current = "builtin"
builtinName = "gentle-bell"
volume = 0.7

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
enabled = true
durationS = 300
postponedS = 600
maxPostponeCount = 2
strictMode = false
afterMiniBreaks = 4

[schedules.longBreaks.theme]
background = { current = "imagePath", imagePath = "/path/to/background.jpg" }
textColor = "#ffffff"
blurRadius = 10
opacity = 0.85
fontSize = 28
fontFamily = "Helvetica"

[schedules.longBreaks.audio]
current = "filePath"
filePath = "/path/to/sound.mp3"
volume = 0.8

[schedules.longBreaks.suggestions]
show = true
```

---

## Attention Settings

Attention reminders are time-based notifications that work like alarm clocks. Unlike breaks, they don't interrupt your work — just show a brief message at specific times, which can be dismissed immediately.

### Basic Attention Fields

Every field of an attention entry is required in the file; the values listed below are the ones the settings UI uses for newly created reminders. A missing field makes the whole configuration fail to load.

#### `id`
- **Type**: Integer
- **Required**: Yes
- **Description**: Internal identifier of this reminder. Ids must be unique among all attention ids — with duplicates, the first matching entry is used. Should not be modified manually!

#### `name`
- **Type**: String
- **Required**: Yes
- **Description**: Name of the reminder (e.g., "Water Reminder", "Stand Up Alert"). New reminders use `"New attention"`.

#### `enabled`
- **Type**: Boolean
- **Required**: Yes
- **Description**: Whether this reminder is active. New reminders use `true`.

#### `times`
- **Type**: Array of strings (24-hour time format)
- **Required**: Yes
- **Description**: List of times when the reminder should trigger

#### `daysOfWeek`
- **Type**: Array of strings
- **Required**: Yes
- **Options**: `"Mon"`, `"Tue"`, `"Wed"`, `"Thu"`, `"Fri"`, `"Sat"`, `"Sun"`
- **Description**: Days when this reminder is active. New reminders use all seven days.

#### `title`
- **Type**: String
- **Required**: Yes
- **Description**: Title shown in the popup window. New reminders use `"Attention Reminder"`.

#### `message`
- **Type**: String
- **Required**: Yes
- **Description**: The message content to display. New reminders use `"This is an attention reminder."`.

#### `durationS`
- **Type**: Integer (seconds)
- **Required**: Yes
- **Description**: How long to show the reminder popup. New reminders use `5`.

#### `theme`
- **Type**: Table
- **Required**: Yes
- **Description**: Appearance of the reminder window. See [Theme Settings](#theme-settings).

**Example Attentions:**
```toml
# Water reminder
[[attentions]]
id = 0
name = "Hydrate"
enabled = true
times = ["10:00", "14:00", "16:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri"]
title = "Time to Drink Water"
message = "Stay hydrated! Drink a glass of water."
durationS = 10

[attentions.theme]
background = { current = "solid", solid = "#0ea5e9" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 20
fontFamily = "Arial"

# Stand up reminder
[[attentions]]
id = 1
name = "Stand Up"
enabled = true
times = ["11:00", "15:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
title = "Stand Up and Move"
message = "Take a moment to stand up and stretch your legs!"
durationS = 15

[attentions.theme]
background = { current = "solid", solid = "#10b981" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 20
fontFamily = "Arial"
```

---

## Theme Settings

Theme settings control the appearance of break windows.

### `background`
- **Type**: Object
- **Description**: Background source for the break window

**Structure:**

```toml
[schedules.miniBreaks.theme.background]
current = "solid"                # Currently active background type
solid = "#1f2937"                # Preserved even when switched to image
imagePath = "/path/to/image.jpg" # Preserved even when switched to folder
imageFolder = "/path/to/images/" # Preserved even when switched to solid
```

**Background Type Options:**

1. **Solid Color** (`current = "solid"`):
   ```toml
   [schedules.miniBreaks.theme.background]
   current = "solid"
   solid = "#1f2937"
   ```

2. **Single Image** (`current = "imagePath"`):
   ```toml
   [schedules.miniBreaks.theme.background]
   current = "imagePath"
   imagePath = "/path/to/image.jpg"
   ```
   - Supported Formats: JPG, PNG, WebP, etc.

3. **Random Image from Folder** (`current = "imageFolder"`):
   ```toml
   [schedules.miniBreaks.theme.background]
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

Pick one of the following — they cannot be combined in the same file.

```toml
# Minimal dark theme
[schedules.miniBreaks.theme]
textColor = "#e2e8f0"
blurRadius = 0
opacity = 1.0
fontSize = 20
fontFamily = "Segoe UI"

[schedules.miniBreaks.theme.background]
current = "solid"
solid = "#0f172a"
```

```toml
# Nature theme with image
[schedules.miniBreaks.theme]
textColor = "#ffffff"
blurRadius = 12
opacity = 0.75
fontSize = 28
fontFamily = "Georgia"

[schedules.miniBreaks.theme.background]
current = "imagePath"
imagePath = "/path/to/forest.jpg"
```

```toml
# Vibrant theme with preserved settings
[schedules.miniBreaks.theme]
textColor = "#fef3c7"
blurRadius = 0
opacity = 0.95
fontSize = 26
fontFamily = "Tahoma"

[schedules.miniBreaks.theme.background]
current = "solid"
solid = "#7c3aed"
imagePath = "/path/to/sunset.jpg" # Preserved
imageFolder = "C:\\Wallpapers"    # Preserved
```

---

## Audio Settings

Audio settings control sound playback during breaks.

> **Note**: The same audio plays when a break **starts** and when it **naturally ends**, helping you stay aware of both transitions.

**Structure**:

```toml
[schedules.miniBreaks.audio]
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

Pick one of the following — they cannot be combined in the same file.

```toml
# No sound
[schedules.miniBreaks.audio]
current = "none"
volume = 0.6
```

```toml
# Built-in sound
[schedules.miniBreaks.audio]
current = "builtin"
builtinName = "gentle-bell"
volume = 0.7
```

```toml
# Custom audio file
[schedules.miniBreaks.audio]
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
suggestions = [] # Legacy compatibility placeholder; not used as a shared pool.
shortSuggestions = [
    "Look away from your screen and focus on a distant object.",
    "Roll your shoulders back and take a deep breath.",
    "Blink slowly and relax your eyes.",
    ...
]
longSuggestions = [
    "Drink a glass of water.",
    "Stand up and stretch your legs.",
    ...
]

[byLanguage.zh-CN]
suggestions = [] # Legacy compatibility placeholder; not used as a shared pool.
shortSuggestions = [
    "将目光从屏幕移开，专注于远处的物体。",
    "向后转动肩膀，深呼吸。",
    "慢慢眨眼，放松眼睛。",
    ...
]
longSuggestions = [
    "喝一杯水。",
    "站起来伸展腿部。",
    ...
]
```

You can add your own suggestions under the appropriate language section. Mini breaks use `shortSuggestions`; long breaks use `longSuggestions`.

`suggestions` is retained only so older Focust versions can still parse the file, and new saves keep it empty. This is parse-only compatibility: older versions that read only `suggestions` may show no suggestions after a newer version has saved the file. When loading an older file that only has `suggestions`, current Focust uses that legacy list as the fallback for any missing short or long pool. If `shortSuggestions = []` or `longSuggestions = []` is present explicitly, that pool is intentionally empty and does not fall back to `suggestions`.

---

## Application Exclusions Settings

Application exclusions let you control when reminders are paused based on which applications are running. This is useful for preventing break interruptions during presentations, video calls, or when using specific applications.

Rules are checked every 10 seconds, and only the **first active rule** is evaluated — later rules are ignored. Rule changes apply after the next app restart. Checks are skipped while a break or reminder window is showing, and an exclusion pause is not cleared by the UI's Resume action.

### `appExclusions`
- **Type**: Array of objects
- **Default**: `[]` (empty array)
- **Description**: List of exclusion rules

### Exclusion Rule Structure

Each exclusion rule has the following structure:

```toml
[[appExclusions]]
active = true
rule = "pause"  # or "resume"
processes = ["chrome.exe", "firefox"]
```

#### `active`
- **Type**: Boolean
- **Default**: `false`
- **Description**: Whether this exclusion rule is active

#### `rule`
- **Type**: String enum
- **Default**: `"pause"`
- **Options**: `"pause"`, `"resume"` (lowercase only)
- **Description**: What happens while the listed processes are running
  - `"pause"`: Pause reminders while any of the processes in `processes` is running
  - `"resume"`: Pause reminders while none of the processes in `processes` is running, i.e. reminders only run while at least one of them is running

#### `processes`
- **Type**: Array of strings
- **Default**: `[]` (empty array)
- **Description**: List of process names or paths to match. Matching is case-insensitive and uses substring matching against the process name and its full executable path.
  - Can be process names (e.g., `"chrome.exe"`, `"firefox"`)
  - Can be partial names (e.g., `"chrome"` matches `"chrome.exe"`)
  - Can be full paths (e.g., `"C:\\Program Files\\App\\app.exe"`)
  - A rule with an empty list never matches: with `rule = "pause"` it does nothing, but with `rule = "resume"` reminders stay paused permanently

### Example Configurations

**Pause during presentations:**
```toml
[[appExclusions]]
active = true
rule = "pause"
processes = ["POWERPNT.EXE", "Keynote", "impress"]  # PowerPoint, Keynote, LibreOffice Impress
```

**Pause during video calls:**
```toml
[[appExclusions]]
active = true
rule = "pause"
processes = ["zoom.exe", "Teams.exe", "Slack.exe", "Discord.exe"]
```

**Reminders only while an editor is running:**
```toml
[[appExclusions]]
active = true
rule = "resume"
processes = ["code.exe", "idea64.exe", "pycharm64.exe"]
```

**Multi-platform example:**
```toml
[[appExclusions]]
active = true
rule = "pause"
processes = [
    # Windows
    "POWERPNT.EXE", "chrome.exe", "firefox.exe",
    # macOS
    "Keynote", "Google Chrome", "Firefox",
    # Linux
    "chrome", "firefox", "libreoffice"
]
```

### Platform-Specific Notes

- **Windows**: Process names typically include `.exe` extension (e.g., `"chrome.exe"`)
- **macOS**: Application names match the `.app` name without extension (e.g., `"Google Chrome"`)
- **Linux**: Process names are typically lowercase without extensions (e.g., `"chrome"`, `"firefox"`)

**Example:**
```toml
autostart = false
monitorDnd = true
inactiveS = 300
allScreens = false
language = "en-US"
themeMode = "system"
postponeShortcut = "Ctrl+Shift+P"
windowSize = 0.8

[[appExclusions]]
active = true
rule = "pause"
processes = ["POWERPNT.EXE", "zoom.exe", "Teams.exe"]

[[appExclusions]]
active = true
rule = "resume"
processes = ["code.exe", "idea64.exe"]
```

---

## Advanced Settings

Advanced settings are intended for users who want to fine-tune the application's behavior beyond standard options. These settings are not exposed in the user interface and must be edited directly in the configuration file. All advanced settings are grouped under the `[advanced]` table.

Advanced settings have a weaker compatibility guarantee than normal settings. Future incompatible changes will be surfaced in the settings interface to notify you of an update, but these options remain intended for manual configuration rather than regular UI workflows.

### `logLevel`
- **Type**: String enum
- **Default**: `"info"` in release builds, `"trace"` in debug builds
- **Options**: `"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"`
- **Description**: Sets the verbosity of application logging. Higher levels provide more detailed logs, useful for troubleshooting.

Example:

```toml
[advanced]
logLevel = "debug" # Enable detailed debug logging
```

### `resetMiniBreakCounterOnPauseReasons`
- **Type**: Array of pause reason strings
- **Default**: `[]`
- **Options**: `"userIdle"`, `"dnd"`, `"manual"`, `"timedManual"`, `"appExclusion"`
- **Description**: Resets the mini-break counter for long-break cadence when the scheduler pauses for any listed reason. For example, adding `"userIdle"` prevents a long break from being triggered by mini breaks counted before you were away.

Example:

```toml
[advanced]
resetMiniBreakCounterOnPauseReasons = ["userIdle"]
```

### `resetOnWake`
- **Type**: Boolean
- **Default**: `true`
- **Description**: Control whether to close the old break window, restart the break interval, and clear the mini-break count used for long breaks, when the system wakes from sleep or hibernation.

```toml
[advanced]
resetOnWake = true
```

### `trayPauseDurationsMinutes`
- **Type**: Array of positive integers
- **Default**: `[15, 30, 60]`
- **Description**: Controls the timed pause durations shown in the system tray menu, in minutes. Selecting one pauses both break and attention reminders until the duration expires. Entries are sorted ascending and deduplicated, and `0` entries are ignored; negative values are invalid and make the whole configuration fail to load. Set this to an empty array (`[]`) to hide the timed pause submenu. Requires application restart to update the tray menu after editing.

Example:

```toml
[advanced]
trayPauseDurationsMinutes = [15, 30, 60]
```

---

## Examples

### Example 1: Minimal Configuration

```toml
# General settings
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
maxPostponeCount = 2
strictMode = false
intervalS = 1200

[schedules.miniBreaks.theme]
textColor = "#f8fafc"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"

[schedules.miniBreaks.theme.background]
current = "solid"
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
maxPostponeCount = 2
strictMode = false
afterMiniBreaks = 4

[schedules.longBreaks.theme]
textColor = "#f8fafc"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"

[schedules.longBreaks.theme.background]
current = "solid"
solid = "#1f2937"

[schedules.longBreaks.audio]
current = "none"
volume = 0.6

[schedules.longBreaks.suggestions]
show = true
```

### Example 2: Comprehensive Work Setup

```toml
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
maxPostponeCount = 2
strictMode = false
intervalS = 1200  # 20 min

[schedules.miniBreaks.theme]
background = { current = "imagePath", imagePath = "/home/user/wallpapers/calm-lake.jpg" }
textColor = "#e0f2fe"
blurRadius = 10
opacity = 0.85
fontSize = 24
fontFamily = "Helvetica"

[schedules.miniBreaks.audio]
current = "builtin"
builtinName = "gentle-bell"
volume = 0.6

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
enabled = true
durationS = 600  # 10 min
postponedS = 600
maxPostponeCount = 2
strictMode = true
afterMiniBreaks = 3

[schedules.longBreaks.theme]
background = { current = "imagePath", imagePath = "/home/user/wallpapers/mountain.jpg" }
textColor = "#ffffff"
blurRadius = 15
opacity = 0.8
fontSize = 28
fontFamily = "Georgia"

[schedules.longBreaks.audio]
current = "filePath"
filePath = "/home/user/sounds/meditation-bell.mp3"
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
maxPostponeCount = 2
strictMode = false
intervalS = 1800  # 30 min

[schedules.miniBreaks.theme]
background = { current = "solid", solid = "#1e3a8a" }
textColor = "#dbeafe"
blurRadius = 5
opacity = 0.9
fontSize = 22
fontFamily = "Arial"

[schedules.miniBreaks.audio]
current = "builtin"
builtinName = "soft-gong"
volume = 0.5

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
enabled = false

# Hydration reminder
[[attentions]]
id = 0
name = "Drink Water"
enabled = true
times = ["10:00", "14:00", "16:00", "20:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
title = "Hydration Time"
message = "Remember to drink water! Stay hydrated throughout the day."
durationS = 10

[attentions.theme]
background = { current = "solid", solid = "#0ea5e9" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 20
fontFamily = "Arial"

# Eye exercise reminder
[[attentions]]
id = 1
name = "Eye Exercise"
enabled = true
times = ["11:00", "15:00", "19:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri"]
title = "Eye Care"
message = "Look at something 20 feet away for 20 seconds (20-20-20 rule)"
durationS = 25

[attentions.theme]
background = { current = "solid", solid = "#10b981" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 22
fontFamily = "Arial"
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
