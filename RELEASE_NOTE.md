# Version 0.2.4

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

- Support restart from tray icon menu.

## 🐛 Bug Fixes

- Fix regression issue that pause/resume in tray icon doesn't work.
- Fix regression issue that DND monitor doesn't work.
- Fix inconsistent state between frontend and scheduler when pause reasons is changed in complicated ways.

## 🚀 Improvements

- Monitors no longer send pause command when in break or attention session.
- Prevent user from manually triggering, postponing or skipping events, when the scheduler is paused.

## 📝 Documentation

- Update QUICKSTART documentation to include FAQ about scheduler pause/resume.