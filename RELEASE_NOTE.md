# Version 0.1.4

##  Improvements

- Supports retaining old values when switching background and audio settings

## 📝 Documentation

- Updates related infomation in CONFIGURATION.md

## ⚠️ Breaking Changes

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

---

**Full Changelog**: [View on GitHub](https://github.com/pilgrimlyieu/Focust/compare/v0.1.3..v0.1.4)
