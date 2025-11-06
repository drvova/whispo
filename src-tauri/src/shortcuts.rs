use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use std::str::FromStr;

pub fn register_global_shortcuts(app: &AppHandle) -> Result<(), String> {
    // Get config to determine shortcut
    // For now, register a default shortcut that can be changed via config

    // Default: Ctrl (Windows/Linux) or Command (macOS) for push-to-talk
    #[cfg(target_os = "macos")]
    let default_shortcut = "CommandOrControl";

    #[cfg(not(target_os = "macos"))]
    let default_shortcut = "Control";

    register_recording_shortcut(app, default_shortcut)?;

    Ok(())
}

pub fn register_recording_shortcut(app: &AppHandle, shortcut_str: &str) -> Result<(), String> {
    // Parse shortcut string
    let shortcut = Shortcut::from_str(shortcut_str)
        .map_err(|e| format!("Invalid shortcut: {}", e))?;

    // Register the shortcut
    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
        match event.state {
            ShortcutState::Pressed => {
                // Shortcut key pressed - start recording
                if let Some(window) = app.get_window("panel") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("shortcut-pressed", ());
                }
            }
            ShortcutState::Released => {
                // Shortcut key released - stop recording
                if let Some(window) = app.get_window("panel") {
                    let _ = window.emit("shortcut-released", ());
                }
            }
        }
    }).map_err(|e| format!("Failed to register shortcut: {}", e))?;

    Ok(())
}

pub fn unregister_all_shortcuts(app: &AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| format!("Failed to unregister shortcuts: {}", e))?;

    Ok(())
}

pub fn update_recording_shortcut(
    app: &AppHandle,
    new_shortcut: &str,
) -> Result<(), String> {
    // Unregister old shortcuts
    unregister_all_shortcuts(app)?;

    // Register new shortcut
    register_recording_shortcut(app, new_shortcut)?;

    Ok(())
}
