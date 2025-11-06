// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use tauri::{
    AppHandle, Manager, State, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, Window, Wry
};
use serde::{Deserialize, Serialize};

mod keyboard;
mod config;
mod commands;

use keyboard::{start_keyboard_listener, write_text};
use config::ConfigStore;

// Application state
#[derive(Default)]
struct AppState {
    is_recording: Arc<Mutex<bool>>,
    config_store: Arc<Mutex<ConfigStore>>,
}

// ===== Tauri Commands =====

#[tauri::command]
async fn restart_app(app: AppHandle) {
    app.restart();
}

#[tauri::command]
async fn show_panel_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_window("panel") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn hide_panel_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_window("panel") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn resize_statusbar_window(
    app: AppHandle,
    width: f64,
    height: f64,
    expanded: Option<bool>,
) -> Result<(), String> {
    if let Some(window) = app.get_window("statusbar") {
        use tauri::Position;

        // Get current display
        let monitor = window.current_monitor().map_err(|e| e.to_string())?;
        if let Some(monitor) = monitor {
            let work_area = monitor.size();
            let scale_factor = monitor.scale_factor();

            // Calculate position
            let x = ((work_area.width as f64 / scale_factor) - width) / 2.0;
            let y = if expanded.unwrap_or(false) {
                (work_area.height as f64 / scale_factor) - height - 80.0
            } else {
                (work_area.height as f64 / scale_factor) - height - 4.0
            };

            window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                width,
                height,
            })).map_err(|e| e.to_string())?;

            window.set_position(Position::Logical(tauri::LogicalPosition { x, y }))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
async fn write_text_command(text: String) -> Result<(), String> {
    write_text(&text).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let config_store = state.config_store.lock().unwrap();
    Ok(config_store.get())
}

#[tauri::command]
async fn save_config(
    state: State<'_, AppState>,
    config: serde_json::Value,
) -> Result<(), String> {
    let mut config_store = state.config_store.lock().unwrap();
    config_store.save(config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn record_event(
    state: State<'_, AppState>,
    event_type: String,
) -> Result<(), String> {
    let mut is_recording = state.is_recording.lock().unwrap();
    *is_recording = event_type == "start";
    Ok(())
}

#[tauri::command]
async fn get_recording_state(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let is_recording = state.is_recording.lock().unwrap();
    Ok(serde_json::json!({
        "isRecording": *is_recording,
    }))
}

#[tauri::command]
async fn display_error(title: Option<String>, message: String) -> Result<(), String> {
    use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder};
    // Note: This needs to be called with app handle
    // For now, we'll just log it
    eprintln!("Error - {}: {}", title.unwrap_or_else(|| "Error".to_string()), message);
    Ok(())
}

// File system operations for recordings
#[tauri::command]
async fn get_recording_history(app: AppHandle) -> Result<Vec<serde_json::Value>, String> {
    use std::fs;
    use std::path::PathBuf;

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let recordings_folder = app_data_dir.join("recordings");
    let history_file = recordings_folder.join("history.json");

    if history_file.exists() {
        let content = fs::read_to_string(history_file).map_err(|e| e.to_string())?;
        let mut history: Vec<serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;

        // Sort by createdAt descending
        history.sort_by(|a, b| {
            let a_time = a["createdAt"].as_i64().unwrap_or(0);
            let b_time = b["createdAt"].as_i64().unwrap_or(0);
            b_time.cmp(&a_time)
        });

        Ok(history)
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
async fn delete_recording_item(app: AppHandle, id: String) -> Result<(), String> {
    use std::fs;

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let recordings_folder = app_data_dir.join("recordings");
    let history_file = recordings_folder.join("history.json");

    if history_file.exists() {
        let content = fs::read_to_string(&history_file).map_err(|e| e.to_string())?;
        let mut history: Vec<serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;

        history.retain(|item| item["id"].as_str() != Some(&id));

        fs::write(&history_file, serde_json::to_string(&history).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

        // Delete the audio file
        let audio_file = recordings_folder.join(format!("{}.webm", id));
        if audio_file.exists() {
            fs::remove_file(audio_file).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
async fn delete_recording_history(app: AppHandle) -> Result<(), String> {
    use std::fs;

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let recordings_folder = app_data_dir.join("recordings");

    if recordings_folder.exists() {
        fs::remove_dir_all(recordings_folder).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn create_recording(
    app: AppHandle,
    recording: Vec<u8>,
    duration: f64,
    use_fusion: Option<bool>,
) -> Result<serde_json::Value, String> {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let recordings_folder = app_data_dir.join("recordings");
    fs::create_dir_all(&recordings_folder).map_err(|e| e.to_string())?;

    // For now, return a simple transcript
    // In full implementation, this would call transcription APIs
    let transcript = "Transcription pending - implement STT integration".to_string();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let id = timestamp.to_string();

    // Save recording file
    let audio_file = recordings_folder.join(format!("{}.webm", id));
    fs::write(audio_file, recording).map_err(|e| e.to_string())?;

    // Update history
    let history_file = recordings_folder.join("history.json");
    let mut history: Vec<serde_json::Value> = if history_file.exists() {
        let content = fs::read_to_string(&history_file).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };

    let item = serde_json::json!({
        "id": id,
        "createdAt": timestamp,
        "duration": duration,
        "transcript": transcript,
        "isOriginalShown": false,
    });

    history.push(item.clone());

    fs::write(&history_file, serde_json::to_string(&history).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    // Copy to clipboard
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard().write_text(transcript.clone()).map_err(|e| e.to_string())?;

    // Write text if accessibility is granted
    let _ = write_text(&transcript);

    // Hide panel window
    if let Some(window) = app.get_window("panel") {
        let _ = window.hide();
    }

    Ok(serde_json::json!({
        "transcript": transcript,
        "fusionResult": null,
    }))
}

fn create_system_tray() -> SystemTray {
    let quit = SystemTrayMenuItem::new("Quit", false, None);
    let show = SystemTrayMenuItem::new("Show", false, None);

    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    SystemTray::new().with_menu(tray_menu)
}

fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            if let Some(window) = app.get_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "quit" => {
                    app.exit(0);
                }
                "show" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn main() {
    // Initialize keyboard listener in a separate thread
    std::thread::spawn(|| {
        if let Err(e) = start_keyboard_listener() {
            eprintln!("Failed to start keyboard listener: {:?}", e);
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState::default())
        .system_tray(create_system_tray())
        .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            restart_app,
            show_panel_window,
            hide_panel_window,
            resize_statusbar_window,
            write_text_command,
            get_config,
            save_config,
            record_event,
            get_recording_state,
            display_error,
            get_recording_history,
            delete_recording_item,
            delete_recording_history,
            create_recording,
        ])
        .setup(|app| {
            // Show appropriate window based on permissions
            // For now, show main window
            if let Some(window) = app.get_window("main") {
                let _ = window.show();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
