# Version 0.2.6

> [!WARNING]
>
> This is a **BETA** version. If you encountered problems, feel free to [open an issue](https://github.com/pilgrimlyieu/Focust/issues/new), along with log information. The log directory is as follows, or you can open it from Advanced Options panel:
> - **Windows**: `%LOCALAPPDATA%\com.fesmoph.focust\logs`
> - **macOS**: `~/Library/Logs/com.fesmoph.focust`
> - **Linux**: `~/.local/share/com.fesmoph.focust/logs`

> [!WARNING]
>
> Audio feature doesn't work in macOS. This is a known upstream issue and will be fixed if its new version is released.

## 🎉 Features

- Support advanced user configuration. Now user can change log level for troubleshooting purpose. (Firstly mentioned in 0.2.5)

## 🐛 Bug Fixes

- Fix some potential deadlock issues in communication between frontend and backend on windows closure.
- Fix reset toast message mistake in settings window.
- Remove transparent effect in break/attention window.

## 🚀 Improvements

- Use the monitor where the cursor is instead of the primary one, when `allScreens` option is disabled.
- Prevent resizing, maximizing, or minimizing the break/attention window.