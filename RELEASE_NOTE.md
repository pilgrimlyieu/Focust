# Version 0.2.0

> [!WARNING]
>
> This is a **BETA** version. If you encountered problems, feel free to [open an issue](https://github.com/pilgrimlyieu/Focust/issues/new).

> [!WARNING]
>
> Audio feature doesn't work in macOS. This is a known upstream issue and will be fixed if its new version is released.

## 🎉 Features

- Add DND(Do Not Disturb) mode detection on Windows, Linux, and macOS to automatically pause break reminders when Focus Assist or equivalent mode is active.
  - Windows uses WNF API, Linux uses D-Bus, macOS uses polling. Welcomes users to enable it via `monitor_dnd` setting and provide feedback.
- Add App Exclusion feature to whitelist applications that temporarily disable break reminders when they are in the foreground or based on more complex rules.
  - Users can configure excluded apps via `excluded_apps` setting in the config file.

## 🚀 Improvements

- Pause and Resume action now work for Attention Timer too.
- Use [user-Idle2](https://crates.io/crates/user-idle2) instead of [user-Idle](https://crates.io/crates/user-idle) for Linux Wayland idle detection.
- Hide debug section in advanced settings panel.

## 📝 Documentation

- Update related documentation for DND monitoring and App Exclusion features.

---

**Full Changelog**: https://github.com/pilgrimlyieu/Focust/compare/v0.1.4...v0.2.0
