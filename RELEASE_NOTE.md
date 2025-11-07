# Version 0.2.2

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

## 🐛 Bug Fixes

- Fix critical bug that when prompt window is set to fullscreen, DND mode will be enabled (this's Focus Assist default behavior on Windows), which causes scheduler pause immediately.

## 🚀 Improvements

- DND feature will not cause panic on Windows platform anymore. (Firstly mentioned in 0.2.1)