# Version 0.2.3

> [!WARNING]
>
> This is a **BETA** version. If you encountered problems, feel free to [open an issue](https://github.com/pilgrimlyieu/Focust/issues/new).

> [!WARNING]
>
> Audio feature doesn't work in macOS. This is a known upstream issue and will be fixed if its new version is released.

> [!WARNING]
>
> DND support is an unstable feature. If you encounter any problems, please feel free to [open an issue](https://github.com/pilgrimlyieu/Focust/issues/new) to provide feedback, along with log information. The log directory is as follows, or you can open it from Advanced Options panel:
> - **Windows**: `%LOCALAPPDATA%\com.fesmoph.focust\logs`
> - **macOS**: `~/Library/Logs/com.fesmoph.focust`
> - **Linux**: `~/.local/share/com.fesmoph.focust/logs`

## 🎉 Features

- Support showing debug section in Advanced Options panel.
- Add `maxPostponeCount` setting to limit the maximum number of postpones for a break.

## 🐛 Bug Fixes

- Fix the issue that nested vacant settings is not fallback to default values.
- Fix the issue that postpone behavior doesn't meet user expectation.

## 🚀 Improvements

- Make error logs when loading configuration fails more accurate.

## 📝 Documentation

- Update related documentation for new `maxPostponeCount` setting.