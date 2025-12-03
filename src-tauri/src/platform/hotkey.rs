//! Global shortcut registration and management.
//!
//! This module handles registering and parsing keyboard shortcuts for
//! application-wide hotkeys, particularly for postponing breaks.

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::cmd::SchedulerCmd;
use crate::config::SharedConfig;
use crate::scheduler::models::Command;

/// Registers global shortcuts for the application.
///
/// Reads the postpone shortcut from config and registers it if specified.
/// If the shortcut string is empty, no shortcut will be registered.
///
/// # Errors
///
/// Returns an error if:
/// - Registering the shortcut fails
pub async fn register_shortcuts<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let postpone_shortcut = {
        let config_state = app.state::<SharedConfig>();
        let config = config_state.read().await;
        config.postpone_shortcut.clone()
    };

    // Only register if shortcut is configured
    if postpone_shortcut.is_empty() {
        tracing::info!("No postpone shortcut configured, skipping registration");
    } else {
        register_postpone_shortcut(app, &postpone_shortcut).map_err(|e| {
            tracing::error!("Failed to register postpone shortcut '{postpone_shortcut}': {e}");
            e
        })?;
        tracing::info!("Global shortcuts registered successfully");
    }

    Ok(())
}

/// Registers the postpone break shortcut.
///
/// # Errors
///
/// Returns an error if:
/// - Parsing the shortcut string fails
/// - Registering the shortcut with the system fails
fn register_postpone_shortcut<R: Runtime>(
    app: &AppHandle<R>,
    shortcut_str: &str,
) -> Result<(), String> {
    // Parse shortcut string (e.g., "Ctrl+Shift+X")
    let shortcut = parse_shortcut(shortcut_str)?;

    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                tracing::debug!("Postpone shortcut triggered");

                // Send postpone command to scheduler
                if let Some(scheduler_cmd) = app_handle.try_state::<SchedulerCmd>() {
                    scheduler_cmd.try_send_command(&Command::PostponeBreak);
                } else {
                    tracing::warn!("SchedulerCmd state not found");
                }
            }
        })
        .map_err(|e| format!("Failed to register shortcut: {e}"))?;

    tracing::info!("Registered postpone shortcut: {shortcut_str}");
    Ok(())
}

/// Parses a shortcut string into a `Shortcut` struct.
///
/// Supported format: "Ctrl+Shift+X", "Alt+F4", etc.
/// Modifiers: Ctrl/Control, Alt, Shift, Super/Meta/Cmd/Win
///
/// # Errors
///
/// Returns an error if:
/// - The shortcut string is empty
/// - Multiple key codes are specified
/// - No key code is found
/// - An unknown key code is specified
fn parse_shortcut(s: &str) -> Result<Shortcut, String> {
    if s.trim().is_empty() {
        return Err("Empty shortcut string".to_owned());
    }

    let parts: Vec<&str> = s
        .split('+')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();

    if parts.is_empty() {
        return Err("Empty shortcut string".to_owned());
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "super" | "meta" | "cmd" | "win" => modifiers |= Modifiers::SUPER,
            // Parse key code
            key => {
                if key_code.is_some() {
                    return Err(format!("Multiple key codes in shortcut: {s}"));
                }
                key_code = Some(parse_key_code(key)?);
            }
        }
    }

    let key = key_code.ok_or_else(|| "No key code found in shortcut".to_owned())?;

    Ok(Shortcut::new(Some(modifiers), key))
}

/// Parses a key string into a `Code` enum.
///
/// Supports letters (a-z), digits (0-9), function keys (F1-F12),
/// and special keys (Space, Enter, Escape, etc.).
///
/// # Errors
///
/// Returns an error if:
/// - The key string is empty
/// - The key code is unknown or unsupported
fn parse_key_code(s: &str) -> Result<Code, String> {
    let lower = s.to_lowercase();

    // Letters: a-z
    if lower.len() == 1 {
        let ch = lower
            .chars()
            .next()
            .ok_or_else(|| "Empty key string".to_owned())?;
        if ch.is_ascii_lowercase() {
            let key_code = match ch {
                'a' => Code::KeyA,
                'b' => Code::KeyB,
                'c' => Code::KeyC,
                'd' => Code::KeyD,
                'e' => Code::KeyE,
                'f' => Code::KeyF,
                'g' => Code::KeyG,
                'h' => Code::KeyH,
                'i' => Code::KeyI,
                'j' => Code::KeyJ,
                'k' => Code::KeyK,
                'l' => Code::KeyL,
                'm' => Code::KeyM,
                'n' => Code::KeyN,
                'o' => Code::KeyO,
                'p' => Code::KeyP,
                'q' => Code::KeyQ,
                'r' => Code::KeyR,
                's' => Code::KeyS,
                't' => Code::KeyT,
                'u' => Code::KeyU,
                'v' => Code::KeyV,
                'w' => Code::KeyW,
                'x' => Code::KeyX,
                'y' => Code::KeyY,
                'z' => Code::KeyZ,
                _ => unreachable!(),
            };
            return Ok(key_code);
        } else if ch.is_ascii_digit() {
            let key_code = match ch {
                '0' => Code::Digit0,
                '1' => Code::Digit1,
                '2' => Code::Digit2,
                '3' => Code::Digit3,
                '4' => Code::Digit4,
                '5' => Code::Digit5,
                '6' => Code::Digit6,
                '7' => Code::Digit7,
                '8' => Code::Digit8,
                '9' => Code::Digit9,
                _ => unreachable!(),
            };
            return Ok(key_code);
        }
        return Err(format!("Unknown key code: {s}"));
    }

    // Function keys: f1-f12
    if lower.starts_with('f')
        && lower.len() >= 2
        && let Ok(num) = lower[1..].parse::<u8>()
    {
        let key_code = match num {
            1 => Code::F1,
            2 => Code::F2,
            3 => Code::F3,
            4 => Code::F4,
            5 => Code::F5,
            6 => Code::F6,
            7 => Code::F7,
            8 => Code::F8,
            9 => Code::F9,
            10 => Code::F10,
            11 => Code::F11,
            12 => Code::F12,
            _ => return Err(format!("Unknown key code: {s}")),
        };
        return Ok(key_code);
    }

    // Special keys
    match lower.as_str() {
        "space" => Ok(Code::Space),
        "enter" | "return" => Ok(Code::Enter),
        "escape" | "esc" => Ok(Code::Escape),
        "tab" => Ok(Code::Tab),
        "backspace" => Ok(Code::Backspace),
        "delete" | "del" => Ok(Code::Delete),
        "insert" | "ins" => Ok(Code::Insert),
        "home" => Ok(Code::Home),
        "end" => Ok(Code::End),
        "pageup" | "pgup" => Ok(Code::PageUp),
        "pagedown" | "pgdown" => Ok(Code::PageDown),
        // Arrow keys
        "left" => Ok(Code::ArrowLeft),
        "right" => Ok(Code::ArrowRight),
        "up" => Ok(Code::ArrowUp),
        "down" => Ok(Code::ArrowDown),
        _ => Err(format!("Unknown key code: {s}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // parse_shortcut basic tests
    #[test]
    fn parse_shortcut_basic() {
        parse_shortcut("Ctrl+Shift+X").unwrap();
        parse_shortcut("Alt+P").unwrap();
        parse_shortcut("Ctrl+F1").unwrap();
    }

    #[test]
    fn parse_shortcut_case_insensitive() {
        parse_shortcut("ctrl+x").unwrap();
        parse_shortcut("CTRL+X").unwrap();
        parse_shortcut("Alt+a").unwrap();
        parse_shortcut("SHIFT+F2").unwrap();
    }

    #[test]
    fn parse_shortcut_all_modifiers() {
        parse_shortcut("Ctrl+Alt+Shift+X").unwrap();
        parse_shortcut("Super+C").unwrap();
        parse_shortcut("Meta+V").unwrap();
        parse_shortcut("Cmd+Z").unwrap();
        parse_shortcut("Win+D").unwrap();
    }

    #[test]
    fn parse_shortcut_letters() {
        // Test all letters
        for letter in 'a'..='z' {
            let shortcut = format!("Ctrl+{letter}");
            assert!(
                parse_shortcut(&shortcut).is_ok(),
                "Failed to parse: {shortcut}"
            );
        }
    }

    #[test]
    fn parse_shortcut_numbers() {
        // Test all digits
        for num in 0..=9 {
            let shortcut = format!("Ctrl+{num}");
            assert!(
                parse_shortcut(&shortcut).is_ok(),
                "Failed to parse: {shortcut}"
            );
        }
    }

    #[test]
    fn parse_shortcut_function_keys() {
        for i in 1..=12 {
            let shortcut = format!("Ctrl+F{i}");
            assert!(
                parse_shortcut(&shortcut).is_ok(),
                "Failed to parse: {shortcut}",
            );
        }
    }

    #[test]
    fn parse_shortcut_special_keys() {
        parse_shortcut("Ctrl+Space").unwrap();
        parse_shortcut("Ctrl+Enter").unwrap();
        parse_shortcut("Ctrl+Return").unwrap();
        parse_shortcut("Ctrl+Escape").unwrap();
        parse_shortcut("Ctrl+Esc").unwrap();
        parse_shortcut("Ctrl+Tab").unwrap();
        parse_shortcut("Ctrl+Backspace").unwrap();
        parse_shortcut("Ctrl+Delete").unwrap();
        parse_shortcut("Ctrl+Del").unwrap();
        parse_shortcut("Ctrl+Insert").unwrap();
        parse_shortcut("Ctrl+Ins").unwrap();
        parse_shortcut("Ctrl+Home").unwrap();
        parse_shortcut("Ctrl+End").unwrap();
        parse_shortcut("Ctrl+PageUp").unwrap();
        parse_shortcut("Ctrl+PgUp").unwrap();
        parse_shortcut("Ctrl+PageDown").unwrap();
        parse_shortcut("Ctrl+PgDown").unwrap();
    }

    #[test]
    fn parse_shortcut_arrow_keys() {
        parse_shortcut("Ctrl+Left").unwrap();
        parse_shortcut("Ctrl+Right").unwrap();
        parse_shortcut("Ctrl+Up").unwrap();
        parse_shortcut("Ctrl+Down").unwrap();
    }

    #[test]
    fn parse_shortcut_whitespace_handling() {
        parse_shortcut("Ctrl + X").unwrap();
        parse_shortcut(" Ctrl+X ").unwrap();
        parse_shortcut("Ctrl  +  X").unwrap();
    }

    // Error case tests
    #[test]
    fn parse_shortcut_empty_string() {
        let result = parse_shortcut("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty shortcut string");
    }

    #[test]
    fn parse_shortcut_only_modifier() {
        let result = parse_shortcut("Ctrl");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No key code found in shortcut");
    }

    #[test]
    fn parse_shortcut_multiple_modifiers_only() {
        let result = parse_shortcut("Ctrl+Alt+Shift");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No key code found in shortcut");
    }

    #[test]
    fn parse_shortcut_unknown_key() {
        let result = parse_shortcut("Ctrl+InvalidKey");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown key code"));
    }

    #[test]
    fn parse_shortcut_multiple_keys() {
        let result = parse_shortcut("X+Y");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Multiple key codes"));
    }

    #[test]
    fn parse_shortcut_multiple_keys_with_modifier() {
        let result = parse_shortcut("Ctrl+X+Y");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Multiple key codes"));
    }

    // parse_key_code tests
    #[test]
    fn parse_key_code_letters() {
        assert!(matches!(parse_key_code("a"), Ok(Code::KeyA)));
        assert!(matches!(parse_key_code("z"), Ok(Code::KeyZ)));
        assert!(matches!(parse_key_code("m"), Ok(Code::KeyM)));
    }

    #[test]
    fn parse_key_code_numbers() {
        assert!(matches!(parse_key_code("0"), Ok(Code::Digit0)));
        assert!(matches!(parse_key_code("5"), Ok(Code::Digit5)));
        assert!(matches!(parse_key_code("9"), Ok(Code::Digit9)));
    }

    #[test]
    fn parse_key_code_function_keys() {
        assert!(matches!(parse_key_code("f1"), Ok(Code::F1)));
        assert!(matches!(parse_key_code("f6"), Ok(Code::F6)));
        assert!(matches!(parse_key_code("f12"), Ok(Code::F12)));
    }

    #[test]
    fn parse_key_code_special_keys() {
        assert!(matches!(parse_key_code("space"), Ok(Code::Space)));
        assert!(matches!(parse_key_code("enter"), Ok(Code::Enter)));
        assert!(matches!(parse_key_code("return"), Ok(Code::Enter)));
        assert!(matches!(parse_key_code("escape"), Ok(Code::Escape)));
        assert!(matches!(parse_key_code("esc"), Ok(Code::Escape)));
        assert!(matches!(parse_key_code("tab"), Ok(Code::Tab)));
        assert!(matches!(parse_key_code("delete"), Ok(Code::Delete)));
        assert!(matches!(parse_key_code("del"), Ok(Code::Delete)));
    }

    #[test]
    fn parse_key_code_arrow_keys() {
        assert!(matches!(parse_key_code("left"), Ok(Code::ArrowLeft)));
        assert!(matches!(parse_key_code("right"), Ok(Code::ArrowRight)));
        assert!(matches!(parse_key_code("up"), Ok(Code::ArrowUp)));
        assert!(matches!(parse_key_code("down"), Ok(Code::ArrowDown)));
    }

    #[test]
    fn parse_key_code_case_insensitive() {
        assert!(matches!(parse_key_code("A"), Ok(Code::KeyA)));
        assert!(matches!(parse_key_code("F1"), Ok(Code::F1)));
        assert!(matches!(parse_key_code("SPACE"), Ok(Code::Space)));
    }

    #[test]
    fn parse_key_code_invalid() {
        parse_key_code("invalid").unwrap_err();
        parse_key_code("ctrl").unwrap_err(); // Modifier keys should not be treated as keys
        parse_key_code("alt").unwrap_err();
        parse_key_code("").unwrap_err();
    }

    // Edge cases and real-world usage tests
    #[test]
    fn parse_shortcut_common_combinations() {
        // Common shortcut combinations
        parse_shortcut("Ctrl+C").unwrap(); // Copy
        parse_shortcut("Ctrl+V").unwrap(); // Paste
        parse_shortcut("Ctrl+X").unwrap(); // Cut
        parse_shortcut("Ctrl+Z").unwrap(); // Undo
        parse_shortcut("Ctrl+Y").unwrap(); // Redo
        parse_shortcut("Ctrl+S").unwrap(); // Save
        parse_shortcut("Ctrl+P").unwrap(); // Print
        parse_shortcut("Ctrl+F").unwrap(); // Find
        parse_shortcut("Alt+F4").unwrap(); // Close window
        parse_shortcut("Ctrl+Shift+Esc").unwrap(); // Task manager
    }

    #[test]
    fn parse_shortcut_modifier_aliases() {
        // Test modifier key aliases
        let ctrl_x = parse_shortcut("Ctrl+X").unwrap();
        let control_x = parse_shortcut("Control+X").unwrap();
        assert_eq!(format!("{ctrl_x:?}"), format!("{control_x:?}"));

        parse_shortcut("Super+D").unwrap();
        parse_shortcut("Meta+D").unwrap();
        parse_shortcut("Cmd+D").unwrap();
        parse_shortcut("Win+D").unwrap();
    }
}
