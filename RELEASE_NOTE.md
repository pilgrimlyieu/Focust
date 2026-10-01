> [!WARNING]
>
> This is a **BETA** version. If you encountered problems, feel free to [open an issue](https://github.com/pilgrimlyieu/Focust/issues/new), along with log information. The log directory is as follows, or you can open it from Advanced Options panel:
> - **Windows**: `%LOCALAPPDATA%\com.fesmoph.focust\logs`
> - **macOS**: `~/Library/Logs/com.fesmoph.focust`
> - **Linux**: `~/.local/share/com.fesmoph.focust/logs`

<!-- Release notes content starts here -->

## 🎉 Features

- Added configuration schema support. `config.toml` now includes a schema comment at the top.

## 🐛 Bug Fixes

- Fixed a potential hang by avoiding stalled playback queues and isolating blocking commands.

## 🚀 Improvements

- Added a hint for the Windows portable build that system notifications are unavailable because the portable build does not register an application identifier (AppUserModelID).

## 📝 Documentation

- Added [an FAQ entry in QUICKSTART](https://github.com/pilgrimlyieu/Focust/blob/main/docs/QUICKSTART.md#no-notifications-with-the-windows-portable-build) for the Windows portable build giving users a manual workaround.
