use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::cmd::SchedulerCmd;
use crate::config::SharedConfig;
use crate::scheduler::models::Command;

/// Register global shortcuts for the application
///
/// Reads the postpone shortcut from config and registers it if specified.
/// If the shortcut string is empty, no shortcut will be registered.
pub async fn register_shortcuts<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let postpone_shortcut = {
        let config_state = app.state::<SharedConfig>();
        let config = config_state.read().await;
        config.postpone_shortcut.clone()
    };

    // Only register if shortcut is configured
    if !postpone_shortcut.is_empty() {
        if let Err(e) = register_postpone_shortcut(app, &postpone_shortcut) {
            tracing::error!("Failed to register postpone shortcut '{postpone_shortcut}': {e}",);
            return Err(e);
        }
        tracing::info!("Global shortcuts registered successfully");
    } else {
        tracing::info!("No postpone shortcut configured, skipping registration");
    }

    Ok(())
}

/// Register the postpone break shortcut
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
                    if let Err(e) = scheduler_cmd.0.try_send(Command::Postpone) {
                        tracing::error!("Failed to send postpone command: {e}");
                    }
                } else {
                    tracing::warn!("SchedulerCmd state not found");
                }
            }
        })
        .map_err(|e| format!("Failed to register shortcut: {e}"))?;

    tracing::info!("Registered postpone shortcut: {shortcut_str}");
    Ok(())
}

/// Parse a shortcut string into a Shortcut struct
/// Supported format: "Ctrl+Shift+X", etc.
fn parse_shortcut(s: &str) -> Result<Shortcut, String> {
    if s.trim().is_empty() {
        return Err("Empty shortcut string".to_string());
    }

    let parts: Vec<&str> = s
        .split('+')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    if parts.is_empty() {
        return Err("Empty shortcut string".to_string());
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

    let key = key_code.ok_or_else(|| "No key code found in shortcut".to_string())?;

    Ok(Shortcut::new(Some(modifiers), key))
}

/// Parse a key string into a Code enum
fn parse_key_code(s: &str) -> Result<Code, String> {
    let lower = s.to_lowercase();

    // Letters: a-z
    if lower.len() == 1 {
        let ch = lower.chars().next().unwrap();
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
    fn test_parse_shortcut_basic() {
        assert!(parse_shortcut("Ctrl+Shift+X").is_ok());
        assert!(parse_shortcut("Alt+P").is_ok());
        assert!(parse_shortcut("Ctrl+F1").is_ok());
    }

    #[test]
    fn test_parse_shortcut_case_insensitive() {
        assert!(parse_shortcut("ctrl+x").is_ok());
        assert!(parse_shortcut("CTRL+X").is_ok());
        assert!(parse_shortcut("Alt+a").is_ok());
        assert!(parse_shortcut("SHIFT+F2").is_ok());
    }

    #[test]
    fn test_parse_shortcut_all_modifiers() {
        assert!(parse_shortcut("Ctrl+Alt+Shift+X").is_ok());
        assert!(parse_shortcut("Super+C").is_ok());
        assert!(parse_shortcut("Meta+V").is_ok());
        assert!(parse_shortcut("Cmd+Z").is_ok());
        assert!(parse_shortcut("Win+D").is_ok());
    }

    #[test]
    fn test_parse_shortcut_letters() {
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
    fn test_parse_shortcut_numbers() {
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
    fn test_parse_shortcut_function_keys() {
        for i in 1..=12 {
            let shortcut = format!("Ctrl+F{i}");
            assert!(
                parse_shortcut(&shortcut).is_ok(),
                "Failed to parse: {shortcut}",
            );
        }
    }

    #[test]
    fn test_parse_shortcut_special_keys() {
        assert!(parse_shortcut("Ctrl+Space").is_ok());
        assert!(parse_shortcut("Ctrl+Enter").is_ok());
        assert!(parse_shortcut("Ctrl+Return").is_ok());
        assert!(parse_shortcut("Ctrl+Escape").is_ok());
        assert!(parse_shortcut("Ctrl+Esc").is_ok());
        assert!(parse_shortcut("Ctrl+Tab").is_ok());
        assert!(parse_shortcut("Ctrl+Backspace").is_ok());
        assert!(parse_shortcut("Ctrl+Delete").is_ok());
        assert!(parse_shortcut("Ctrl+Del").is_ok());
        assert!(parse_shortcut("Ctrl+Insert").is_ok());
        assert!(parse_shortcut("Ctrl+Ins").is_ok());
        assert!(parse_shortcut("Ctrl+Home").is_ok());
        assert!(parse_shortcut("Ctrl+End").is_ok());
        assert!(parse_shortcut("Ctrl+PageUp").is_ok());
        assert!(parse_shortcut("Ctrl+PgUp").is_ok());
        assert!(parse_shortcut("Ctrl+PageDown").is_ok());
        assert!(parse_shortcut("Ctrl+PgDown").is_ok());
    }

    #[test]
    fn test_parse_shortcut_arrow_keys() {
        assert!(parse_shortcut("Ctrl+Left").is_ok());
        assert!(parse_shortcut("Ctrl+Right").is_ok());
        assert!(parse_shortcut("Ctrl+Up").is_ok());
        assert!(parse_shortcut("Ctrl+Down").is_ok());
    }

    #[test]
    fn test_parse_shortcut_whitespace_handling() {
        assert!(parse_shortcut("Ctrl + X").is_ok());
        assert!(parse_shortcut(" Ctrl+X ").is_ok());
        assert!(parse_shortcut("Ctrl  +  X").is_ok());
    }

    // Error case tests
    #[test]
    fn test_parse_shortcut_empty_string() {
        let result = parse_shortcut("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty shortcut string");
    }

    #[test]
    fn test_parse_shortcut_only_modifier() {
        let result = parse_shortcut("Ctrl");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No key code found in shortcut");
    }

    #[test]
    fn test_parse_shortcut_multiple_modifiers_only() {
        let result = parse_shortcut("Ctrl+Alt+Shift");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No key code found in shortcut");
    }

    #[test]
    fn test_parse_shortcut_unknown_key() {
        let result = parse_shortcut("Ctrl+InvalidKey");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown key code"));
    }

    #[test]
    fn test_parse_shortcut_multiple_keys() {
        let result = parse_shortcut("X+Y");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Multiple key codes"));
    }

    #[test]
    fn test_parse_shortcut_multiple_keys_with_modifier() {
        let result = parse_shortcut("Ctrl+X+Y");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Multiple key codes"));
    }

    // parse_key_code tests
    #[test]
    fn test_parse_key_code_letters() {
        assert!(matches!(parse_key_code("a"), Ok(Code::KeyA)));
        assert!(matches!(parse_key_code("z"), Ok(Code::KeyZ)));
        assert!(matches!(parse_key_code("m"), Ok(Code::KeyM)));
    }

    #[test]
    fn test_parse_key_code_numbers() {
        assert!(matches!(parse_key_code("0"), Ok(Code::Digit0)));
        assert!(matches!(parse_key_code("5"), Ok(Code::Digit5)));
        assert!(matches!(parse_key_code("9"), Ok(Code::Digit9)));
    }

    #[test]
    fn test_parse_key_code_function_keys() {
        assert!(matches!(parse_key_code("f1"), Ok(Code::F1)));
        assert!(matches!(parse_key_code("f6"), Ok(Code::F6)));
        assert!(matches!(parse_key_code("f12"), Ok(Code::F12)));
    }

    #[test]
    fn test_parse_key_code_special_keys() {
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
    fn test_parse_key_code_arrow_keys() {
        assert!(matches!(parse_key_code("left"), Ok(Code::ArrowLeft)));
        assert!(matches!(parse_key_code("right"), Ok(Code::ArrowRight)));
        assert!(matches!(parse_key_code("up"), Ok(Code::ArrowUp)));
        assert!(matches!(parse_key_code("down"), Ok(Code::ArrowDown)));
    }

    #[test]
    fn test_parse_key_code_case_insensitive() {
        assert!(matches!(parse_key_code("A"), Ok(Code::KeyA)));
        assert!(matches!(parse_key_code("F1"), Ok(Code::F1)));
        assert!(matches!(parse_key_code("SPACE"), Ok(Code::Space)));
    }

    #[test]
    fn test_parse_key_code_invalid() {
        assert!(parse_key_code("invalid").is_err());
        assert!(parse_key_code("ctrl").is_err()); // Modifier keys should not be treated as keys
        assert!(parse_key_code("alt").is_err());
        assert!(parse_key_code("").is_err());
    }

    // Edge cases and real-world usage tests
    #[test]
    fn test_parse_shortcut_common_combinations() {
        // Common shortcut combinations
        assert!(parse_shortcut("Ctrl+C").is_ok()); // Copy
        assert!(parse_shortcut("Ctrl+V").is_ok()); // Paste
        assert!(parse_shortcut("Ctrl+X").is_ok()); // Cut
        assert!(parse_shortcut("Ctrl+Z").is_ok()); // Undo
        assert!(parse_shortcut("Ctrl+Y").is_ok()); // Redo
        assert!(parse_shortcut("Ctrl+S").is_ok()); // Save
        assert!(parse_shortcut("Ctrl+P").is_ok()); // Print
        assert!(parse_shortcut("Ctrl+F").is_ok()); // Find
        assert!(parse_shortcut("Alt+F4").is_ok()); // Close window
        assert!(parse_shortcut("Ctrl+Shift+Esc").is_ok()); // Task manager
    }

    #[test]
    fn test_parse_shortcut_modifier_aliases() {
        // Test modifier key aliases
        let ctrl_x = parse_shortcut("Ctrl+X").unwrap();
        let control_x = parse_shortcut("Control+X").unwrap();
        assert_eq!(format!("{ctrl_x:?}"), format!("{control_x:?}"));

        assert!(parse_shortcut("Super+D").is_ok());
        assert!(parse_shortcut("Meta+D").is_ok());
        assert!(parse_shortcut("Cmd+D").is_ok());
        assert!(parse_shortcut("Win+D").is_ok());
    }
}
