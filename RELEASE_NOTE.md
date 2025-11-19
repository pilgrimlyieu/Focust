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

- Add pause reasons display support in the settings UI to inform users why schedulers are paused.
- Add toast notifications when user attempts to resume paused schedulers manually that are still paused due to other reasons.

## 🐛 Bug Fixes

- Fix potential prompt-window-specific freezing issue when audio playback fails silently.

## 🚀 Improvements

- Enhance audio playback commands with timeout handling and logging.