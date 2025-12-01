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

## 🐛 Bug Fixes

- Fix: Older log files are not deleted automatically. This is an upstream issue, and has been fixed in the latest version. (https://github.com/tokio-rs/tracing/pull/2966)