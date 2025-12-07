> [!WARNING]
>
> This is a **BETA** version. If you encountered problems, feel free to [open an issue](https://github.com/pilgrimlyieu/Focust/issues/new), along with log information. The log directory is as follows, or you can open it from Advanced Options panel:
> - **Windows**: `%LOCALAPPDATA%\com.fesmoph.focust\logs`
> - **macOS**: `~/Library/Logs/com.fesmoph.focust`
> - **Linux**: `~/.local/share/com.fesmoph.focust/logs`

> [!WARNING]
>
> Audio feature doesn't work in macOS. This is a known upstream issue and will be fixed if its new version is released.

<!-- Release notes content starts here -->

## 🎉 Features

- Added folding/collapsing functionality (default collapsed) to the breaking schedule and attention settings panel to improve space efficiency and presentation. Dragging functionality has been removed from these panels since it is no longer necessary and does not work.

## 🚀 Improvements

- Replaced the tick mechanism with sleep for dynamic event calculation. This ensures more accurate timing and prevents unintended time compensation.