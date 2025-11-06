// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::{
    AppHandle, Manager, State, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

mod keyboard;
mod config;
mod platform;
mod types;
mod state;
mod shortcuts;
mod mcp;

use config::ConfigStore;
use state::AppState;
use types::*;
use mcp::McpClient;
use std::sync::mpsc::channel;

// ===== CORE TAURI COMMANDS =====

#[tauri::command]
async fn restart_app(app: AppHandle) {
    app.restart();
}

#[tauri::command]
async fn get_update_info() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "available": false,
        "version": env!("CARGO_PKG_VERSION"),
        "releaseNotes": ""
    }))
}

#[tauri::command]
async fn quit_and_install(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
async fn check_for_updates_and_download() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "available": false,
        "downloading": false
    }))
}

// ===== WINDOW MANAGEMENT =====

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

        let monitor = window.current_monitor().map_err(|e| e.to_string())?;
        if let Some(monitor) = monitor {
            let work_area = monitor.size();
            let scale_factor = monitor.scale_factor();

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

// ===== SYSTEM INTEGRATION =====

#[tauri::command]
async fn open_microphone_in_system_preferences() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn show_context_menu(
    app: AppHandle,
    x: i32,
    y: i32,
    selected_text: Option<String>,
) -> Result<(), String> {
    if let Some(text) = selected_text {
        app.clipboard().write_text(text).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn get_microphone_status() -> Result<String, String> {
    Ok(platform::get_microphone_status())
}

#[tauri::command]
async fn is_accessibility_granted() -> Result<bool, String> {
    Ok(platform::is_accessibility_granted())
}

#[tauri::command]
async fn request_accessibility_access() -> Result<bool, String> {
    Ok(platform::request_accessibility_access())
}

#[tauri::command]
async fn request_microphone_access() -> Result<bool, String> {
    Ok(platform::request_microphone_access().await)
}

#[tauri::command]
async fn display_error(app: AppHandle, title: Option<String>, message: String) -> Result<(), String> {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

    app.dialog()
        .message(message)
        .title(title.unwrap_or_else(|| "Error".to_string()))
        .kind(MessageDialogKind::Error)
        .blocking_show();

    Ok(())
}

// ===== KEYBOARD & TEXT =====

#[tauri::command]
async fn write_text_command(text: String) -> Result<(), String> {
    keyboard::write_text(&text).map_err(|e| e.to_string())
}

// ===== RECORDING MANAGEMENT =====

#[tauri::command]
async fn get_recording_history(app: AppHandle) -> Result<Vec<RecordingHistoryItem>, String> {
    use std::fs;

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let recordings_folder = app_data_dir.join("recordings");
    let history_file = recordings_folder.join("history.json");

    if history_file.exists() {
        let content = fs::read_to_string(history_file).map_err(|e| e.to_string())?;
        let mut history: Vec<RecordingHistoryItem> = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;

        history.sort_by(|a, b| b.created_at.cmp(&a.created_at));

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
        let mut history: Vec<RecordingHistoryItem> = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;

        history.retain(|item| item.id != id);

        fs::write(&history_file, serde_json::to_string(&history).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

        let audio_file = recordings_folder.join(format!("{}.webm", id));
        if audio_file.exists() {
            fs::remove_file(audio_file).map_err(|e| e.to_string())?;
        }
    }

    if let Some(window) = app.get_window("main") {
        window.emit("refresh-recording-history", ()).map_err(|e| e.to_string())?;
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
async fn toggle_recording_transcript(
    app: AppHandle,
    id: String,
) -> Result<serde_json::Value, String> {
    use std::fs;

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let recordings_folder = app_data_dir.join("recordings");
    let history_file = recordings_folder.join("history.json");

    if !history_file.exists() {
        return Err("History file not found".to_string());
    }

    let content = fs::read_to_string(&history_file).map_err(|e| e.to_string())?;
    let mut history: Vec<RecordingHistoryItem> = serde_json::from_str(&content)
        .map_err(|e| e.to_string())?;

    if let Some(item) = history.iter_mut().find(|i| i.id == id) {
        if item.original_transcript.is_none() {
            return Err("No original transcript available".to_string());
        }

        let is_showing = item.is_original_shown.unwrap_or(false);
        item.is_original_shown = Some(!is_showing);

        fs::write(&history_file, serde_json::to_string(&history).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

        if let Some(window) = app.get_window("main") {
            window.emit("refresh-recording-history", ()).map_err(|e| e.to_string())?;
        }

        Ok(serde_json::json!({
            "success": true,
            "isShowingOriginal": !is_showing
        }))
    } else {
        Err("Recording not found".to_string())
    }
}

#[tauri::command]
async fn create_recording(
    app: AppHandle,
    config_store: State<'_, Arc<ConfigStore>>,
    recording: Vec<u8>,
    duration: f64,
    use_fusion: Option<bool>,
) -> Result<serde_json::Value, String> {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let recordings_folder = app_data_dir.join("recordings");
    fs::create_dir_all(&recordings_folder).map_err(|e| e.to_string())?;

    let config = config_store.get();
    let blob = recording.clone();
    let transcript = transcribe_audio(&config, blob).await?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let id = timestamp.to_string();

    let audio_file = recordings_folder.join(format!("{}.webm", id));
    fs::write(audio_file, recording).map_err(|e| e.to_string())?;

    let history_file = recordings_folder.join("history.json");
    let mut history: Vec<RecordingHistoryItem> = if history_file.exists() {
        let content = fs::read_to_string(&history_file).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };

    let item = RecordingHistoryItem {
        id: id.clone(),
        created_at: timestamp,
        duration,
        transcript: transcript.clone(),
        original_transcript: None,
        is_original_shown: None,
    };

    history.push(item);

    fs::write(&history_file, serde_json::to_string(&history).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    app.clipboard().write_text(transcript.clone()).map_err(|e| e.to_string())?;

    if platform::is_accessibility_granted() {
        let _ = keyboard::write_text(&transcript);
    }

    if let Some(window) = app.get_window("panel") {
        let _ = window.hide();
    }

    if let Some(window) = app.get_window("main") {
        window.emit("refresh-recording-history", ()).map_err(|e| e.to_string())?;
    }

    Ok(serde_json::json!({
        "transcript": transcript,
        "fusionResult": null,
    }))
}

async fn transcribe_audio(config: &serde_json::Value, audio_data: Vec<u8>) -> Result<String, String> {
    let provider_id = config.get("sttProviderId")
        .and_then(|v| v.as_str())
        .unwrap_or("openai");

    let (api_key, base_url, model) = match provider_id {
        "groq" => (
            config.get("groqApiKey").and_then(|v| v.as_str()).unwrap_or(""),
            config.get("groqBaseUrl").and_then(|v| v.as_str()).unwrap_or("https://api.groq.com/openai/v1"),
            "whisper-large-v3"
        ),
        _ => (
            config.get("openaiApiKey").and_then(|v| v.as_str()).unwrap_or(""),
            config.get("openaiBaseUrl").and_then(|v| v.as_str()).unwrap_or("https://api.openai.com/v1"),
            "whisper-1"
        ),
    };

    if api_key.is_empty() {
        return Err("API key not configured".to_string());
    }

    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(audio_data)
            .file_name("recording.webm")
            .mime_str("audio/webm").unwrap())
        .text("model", model.to_string())
        .text("response_format", "json");

    let response = client
        .post(format!("{}/audio/transcriptions", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Transcription failed: {}", error_text));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let text = json.get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(text)
}

// ===== CONFIG MANAGEMENT =====

#[tauri::command]
async fn get_config(config_store: State<'_, Arc<ConfigStore>>) -> Result<serde_json::Value, String> {
    Ok(config_store.get())
}

#[tauri::command]
async fn save_config(
    config_store: State<'_, Arc<ConfigStore>>,
    config: serde_json::Value,
) -> Result<(), String> {
    config_store.save(config).map_err(|e| e.to_string())
}

// ===== STATE MANAGEMENT =====

#[tauri::command]
async fn record_event(
    app_state: State<'_, Arc<AppState>>,
    event_type: String,
) -> Result<(), String> {
    let mut is_recording = app_state.is_recording.lock().unwrap();
    *is_recording = event_type == "start";
    Ok(())
}

#[tauri::command]
async fn get_recording_state(app_state: State<'_, Arc<AppState>>) -> Result<serde_json::Value, String> {
    let is_recording = app_state.is_recording.lock().unwrap();
    Ok(serde_json::json!({
        "isRecording": *is_recording,
    }))
}

// ===== PROFILES =====

#[tauri::command]
async fn get_profiles(config_store: State<'_, Arc<ConfigStore>>) -> Result<Vec<SettingsProfile>, String> {
    Ok(config_store.get_profiles())
}

#[tauri::command]
async fn get_active_profile_id(config_store: State<'_, Arc<ConfigStore>>) -> Result<Option<String>, String> {
    Ok(config_store.get_active_profile_id())
}

#[tauri::command]
async fn create_profile(
    config_store: State<'_, Arc<ConfigStore>>,
    name: String,
    description: Option<String>,
    base_config: Option<serde_json::Value>,
) -> Result<SettingsProfile, String> {
    config_store.create_profile(name, description, base_config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_profile(
    config_store: State<'_, Arc<ConfigStore>>,
    profile_id: String,
    updates: serde_json::Value,
) -> Result<(), String> {
    config_store.update_profile(profile_id, updates)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_profile(
    config_store: State<'_, Arc<ConfigStore>>,
    profile_id: String,
) -> Result<bool, String> {
    config_store.delete_profile(profile_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn switch_profile(
    config_store: State<'_, Arc<ConfigStore>>,
    profile_id: String,
) -> Result<bool, String> {
    config_store.switch_profile(profile_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn duplicate_profile(
    config_store: State<'_, Arc<ConfigStore>>,
    profile_id: String,
    new_name: String,
) -> Result<Option<SettingsProfile>, String> {
    let profiles = config_store.get_profiles();

    if let Some(profile) = profiles.iter().find(|p| p.id == profile_id) {
        let new_profile = config_store.create_profile(
            new_name,
            Some(format!("Copy of {}", profile.description.as_deref().unwrap_or(&profile.name))),
            Some(profile.config.clone()),
        ).map_err(|e| e.to_string())?;
        Ok(Some(new_profile))
    } else {
        Ok(None)
    }
}

// ===== APP RULES =====

#[tauri::command]
async fn get_active_application(app_state: State<'_, Arc<AppState>>) -> Result<Option<ActiveApplication>, String> {
    let active_app = app_state.active_app.lock().unwrap();
    Ok(active_app.clone())
}

#[tauri::command]
async fn update_active_application(app_state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let app = platform::get_active_application().map_err(|e| e.to_string())?;
    let mut active_app = app_state.active_app.lock().unwrap();
    *active_app = Some(app);
    Ok(())
}

#[tauri::command]
async fn get_effective_config(
    config_store: State<'_, Arc<ConfigStore>>,
    app_state: State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    let config = config_store.get();
    let active_rule = app_state.active_rule.lock().unwrap();

    if let Some(rule) = active_rule.as_ref() {
        let mut effective = config.clone();
        if let Some(obj) = effective.as_object_mut() {
            if let Some(provider) = &rule.stt_provider_id {
                obj.insert("sttProviderId".to_string(), serde_json::Value::String(provider.clone()));
            }
        }
        Ok(effective)
    } else {
        Ok(config)
    }
}

#[tauri::command]
async fn create_app_rule(
    config_store: State<'_, Arc<ConfigStore>>,
    rule: serde_json::Value,
) -> Result<AppRule, String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let mut new_rule: AppRule = serde_json::from_value(rule).map_err(|e| e.to_string())?;
    new_rule.id = format!("rule_{}", timestamp);

    let mut config = config_store.get();
    if let Some(obj) = config.as_object_mut() {
        let mut app_rules: Vec<AppRule> = obj.get("appRules")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        app_rules.push(new_rule.clone());
        obj.insert("appRules".to_string(), serde_json::to_value(&app_rules).unwrap());
    }

    config_store.save(config).map_err(|e| e.to_string())?;
    Ok(new_rule)
}

#[tauri::command]
async fn update_app_rule(
    config_store: State<'_, Arc<ConfigStore>>,
    rule: AppRule,
) -> Result<AppRule, String> {
    let mut config = config_store.get();

    if let Some(obj) = config.as_object_mut() {
        let mut app_rules: Vec<AppRule> = obj.get("appRules")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        if let Some(existing_rule) = app_rules.iter_mut().find(|r| r.id == rule.id) {
            *existing_rule = rule.clone();
        }

        obj.insert("appRules".to_string(), serde_json::to_value(&app_rules).unwrap());
    }

    config_store.save(config).map_err(|e| e.to_string())?;
    Ok(rule)
}

#[tauri::command]
async fn delete_app_rule(
    config_store: State<'_, Arc<ConfigStore>>,
    id: String,
) -> Result<(), String> {
    let mut config = config_store.get();

    if let Some(obj) = config.as_object_mut() {
        let mut app_rules: Vec<AppRule> = obj.get("appRules")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        app_rules.retain(|r| r.id != id);
        obj.insert("appRules".to_string(), serde_json::to_value(&app_rules).unwrap());
    }

    config_store.save(config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_app_rules(config_store: State<'_, Arc<ConfigStore>>) -> Result<Vec<AppRule>, String> {
    let config = config_store.get();
    let app_rules: Vec<AppRule> = config.get("appRules")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    Ok(app_rules)
}

#[tauri::command]
async fn test_app_rule(
    app_state: State<'_, Arc<AppState>>,
    rule: AppRule,
) -> Result<bool, String> {
    let active_app = app_state.active_app.lock().unwrap();

    if let Some(app) = active_app.as_ref() {
        let app_name_matches = app.name.to_lowercase().contains(&rule.app_name.to_lowercase());
        let exe_matches = rule.executable.as_ref()
            .map(|exe| app.executable.to_lowercase().contains(&exe.to_lowercase()))
            .unwrap_or(true);

        Ok(app_name_matches && exe_matches)
    } else {
        Ok(false)
    }
}

// ===== VOICE ACTIVATION =====

#[tauri::command]
async fn init_voice_activation() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
async fn start_voice_activation(app_state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut status = app_state.voice_activation.lock().unwrap();
    status.is_enabled = true;
    status.is_listening = true;
    Ok(())
}

#[tauri::command]
async fn stop_voice_activation(app_state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut status = app_state.voice_activation.lock().unwrap();
    status.is_listening = false;
    Ok(())
}

#[tauri::command]
async fn get_voice_activation_status(app_state: State<'_, Arc<AppState>>) -> Result<VoiceActivationStatus, String> {
    let status = app_state.voice_activation.lock().unwrap();
    Ok(status.clone())
}

#[tauri::command]
async fn cleanup_voice_activation(app_state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut status = app_state.voice_activation.lock().unwrap();
    status.is_enabled = false;
    status.is_listening = false;
    Ok(())
}

// ===== STREAMING DICTATION =====

#[tauri::command]
async fn init_streaming_dictation() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
async fn start_streaming_dictation(app_state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut status = app_state.streaming_dictation.lock().unwrap();
    status.is_active = true;
    status.is_listening = true;
    Ok(())
}

#[tauri::command]
async fn stop_streaming_dictation(app_state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut status = app_state.streaming_dictation.lock().unwrap();
    status.is_active = false;
    status.is_listening = false;
    Ok(())
}

#[tauri::command]
async fn pause_streaming_dictation(app_state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut status = app_state.streaming_dictation.lock().unwrap();
    status.is_listening = false;
    Ok(())
}

#[tauri::command]
async fn resume_streaming_dictation(app_state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut status = app_state.streaming_dictation.lock().unwrap();
    status.is_listening = true;
    Ok(())
}

#[tauri::command]
async fn toggle_streaming_dictation(app_state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut status = app_state.streaming_dictation.lock().unwrap();
    status.is_listening = !status.is_listening;
    Ok(())
}

#[tauri::command]
async fn get_streaming_dictation_status(app_state: State<'_, Arc<AppState>>) -> Result<StreamingDictationStatus, String> {
    let status = app_state.streaming_dictation.lock().unwrap();
    Ok(status.clone())
}

#[tauri::command]
async fn cleanup_streaming_dictation(app_state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut status = app_state.streaming_dictation.lock().unwrap();
    status.is_active = false;
    status.is_listening = false;
    Ok(())
}

// ===== FUSION & CONTEXT =====

#[tauri::command]
async fn test_fusion_configuration() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "success": true,
        "providersAvailable": ["openai", "groq"]
    }))
}

#[tauri::command]
async fn get_fusion_config(config_store: State<'_, Arc<ConfigStore>>) -> Result<serde_json::Value, String> {
    let config = config_store.get();
    Ok(config.get("fusionTranscription").cloned().unwrap_or(serde_json::json!({})))
}

#[tauri::command]
async fn update_fusion_config(
    config_store: State<'_, Arc<ConfigStore>>,
    fusion_config: serde_json::Value,
) -> Result<(), String> {
    let mut config = config_store.get();
    if let Some(obj) = config.as_object_mut() {
        obj.insert("fusionTranscription".to_string(), fusion_config);
    }
    config_store.save(config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn test_context_detection() -> Result<serde_json::Value, String> {
    let app_info = platform::get_active_application().ok();

    Ok(serde_json::json!({
        "appInfo": app_info,
        "detectedContext": "generic",
        "availableContexts": ["code-editor", "terminal", "email", "chat", "document", "browser", "notes", "generic"]
    }))
}

#[tauri::command]
async fn get_current_app_info() -> Result<Option<ActiveApplication>, String> {
    Ok(platform::get_active_application().ok())
}

#[tauri::command]
async fn detect_context_for_app(app_info: ActiveApplication) -> Result<String, String> {
    let context = match app_info.executable.to_lowercase().as_str() {
        name if name.contains("code") || name.contains("studio") => "code-editor",
        name if name.contains("terminal") || name.contains("iterm") || name.contains("cmd") => "terminal",
        name if name.contains("mail") || name.contains("outlook") => "email",
        name if name.contains("slack") || name.contains("discord") || name.contains("teams") => "chat",
        name if name.contains("word") || name.contains("docs") => "document",
        name if name.contains("chrome") || name.contains("firefox") || name.contains("safari") => "browser",
        name if name.contains("notes") || name.contains("notion") => "notes",
        _ => "generic",
    };

    Ok(context.to_string())
}

#[tauri::command]
async fn get_effective_formatting_config() -> Result<Option<serde_json::Value>, String> {
    if let Ok(app) = platform::get_active_application() {
        let context = detect_context_for_app(app).await?;
        Ok(Some(serde_json::json!({
            "context": context,
            "enabled": true
        })))
    } else {
        Ok(None)
    }
}

#[tauri::command]
async fn preview_context_formatting(
    transcript: String,
    formatting_config: serde_json::Value,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "basicFormatted": transcript,
        "aiPrompt": format!("Format this text for {} context: {}",
            formatting_config.get("context").and_then(|v| v.as_str()).unwrap_or("generic"),
            transcript),
        "originalTranscript": transcript
    }))
}

// ===== GLOBAL SHORTCUTS =====

#[tauri::command]
async fn register_shortcut(app: AppHandle, shortcut: String) -> Result<(), String> {
    shortcuts::update_recording_shortcut(&app, &shortcut)
}

#[tauri::command]
async fn unregister_shortcuts(app: AppHandle) -> Result<(), String> {
    shortcuts::unregister_all_shortcuts(&app)
}

// ===== MCP INTEGRATION =====

#[tauri::command]
async fn mcp_initialize(
    mcp_client: State<'_, Arc<McpClient>>,
) -> Result<(), String> {
    mcp_client.initialize().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn mcp_is_enabled(
    mcp_client: State<'_, Arc<McpClient>>,
) -> Result<bool, String> {
    Ok(mcp_client.is_enabled())
}

#[tauri::command]
async fn mcp_get_config(
    mcp_client: State<'_, Arc<McpClient>>,
) -> Result<mcp::McpConfiguration, String> {
    Ok(mcp_client.get_config())
}

#[tauri::command]
async fn mcp_update_config(
    mcp_client: State<'_, Arc<McpClient>>,
    config: mcp::McpConfiguration,
) -> Result<(), String> {
    mcp_client.update_config(config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn mcp_list_tools(
    mcp_client: State<'_, Arc<McpClient>>,
) -> Result<Vec<mcp::McpTool>, String> {
    mcp_client.list_tools().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn mcp_call_tool(
    mcp_client: State<'_, Arc<McpClient>>,
    tool_name: String,
    arguments: std::collections::HashMap<String, serde_json::Value>,
) -> Result<mcp::ToolResult, String> {
    mcp_client.call_tool(&tool_name, arguments).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn mcp_get_context(
    mcp_client: State<'_, Arc<McpClient>>,
) -> Result<mcp::TranscriptionContext, String> {
    mcp_client.get_transcription_context().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn mcp_enhance_transcript(
    mcp_client: State<'_, Arc<McpClient>>,
    transcript: String,
) -> Result<String, String> {
    mcp_client.enhance_transcript(&transcript).await.map_err(|e| e.to_string())
}

// ===== SYSTEM TRAY =====

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

// ===== MAIN =====

fn main() {
    keyboard::init_keyboard_system();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");

            let config_store = Arc::new(
                ConfigStore::new(app_data_dir).expect("Failed to initialize config store")
            );
            let app_state = Arc::new(AppState::new());

            // Initialize MCP client
            let mcp_config = mcp::McpConfiguration::default();
            let mcp_client = Arc::new(McpClient::new(mcp_config));

            app.manage(config_store);
            app.manage(app_state);
            app.manage(mcp_client);

            let app_handle = app.handle();
            let (tx, rx) = channel();
            keyboard::set_event_sender(tx);

            std::thread::spawn(move || {
                while let Ok(event) = rx.recv() {
                    if let Some(window) = app_handle.get_window("main") {
                        let _ = window.emit("keyboard-event", &event);
                    }
                    if let Some(window) = app_handle.get_window("panel") {
                        let _ = window.emit("keyboard-event", &event);
                    }
                }
            });

            std::thread::spawn(|| {
                if let Err(e) = keyboard::start_keyboard_listener() {
                    eprintln!("Failed to start keyboard listener: {:?}", e);
                }
            });

            // Register global shortcuts
            if let Err(e) = shortcuts::register_global_shortcuts(app.handle()) {
                eprintln!("Failed to register shortcuts: {}", e);
            }

            if platform::is_accessibility_granted() {
                if let Some(window) = app.get_window("main") {
                    let _ = window.show();
                }
            } else {
                if let Some(window) = app.get_window("setup") {
                    let _ = window.show();
                }
            }

            Ok(())
        })
        .system_tray(create_system_tray())
        .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            restart_app,
            get_update_info,
            quit_and_install,
            check_for_updates_and_download,
            show_panel_window,
            hide_panel_window,
            resize_statusbar_window,
            open_microphone_in_system_preferences,
            show_context_menu,
            get_microphone_status,
            is_accessibility_granted,
            request_accessibility_access,
            request_microphone_access,
            display_error,
            write_text_command,
            get_recording_history,
            delete_recording_item,
            delete_recording_history,
            toggle_recording_transcript,
            create_recording,
            get_config,
            save_config,
            record_event,
            get_recording_state,
            get_profiles,
            get_active_profile_id,
            create_profile,
            update_profile,
            delete_profile,
            switch_profile,
            duplicate_profile,
            get_active_application,
            update_active_application,
            get_effective_config,
            create_app_rule,
            update_app_rule,
            delete_app_rule,
            get_app_rules,
            test_app_rule,
            init_voice_activation,
            start_voice_activation,
            stop_voice_activation,
            get_voice_activation_status,
            cleanup_voice_activation,
            init_streaming_dictation,
            start_streaming_dictation,
            stop_streaming_dictation,
            pause_streaming_dictation,
            resume_streaming_dictation,
            toggle_streaming_dictation,
            get_streaming_dictation_status,
            cleanup_streaming_dictation,
            test_fusion_configuration,
            get_fusion_config,
            update_fusion_config,
            test_context_detection,
            get_current_app_info,
            detect_context_for_app,
            get_effective_formatting_config,
            preview_context_formatting,
            register_shortcut,
            unregister_shortcuts,
            mcp_initialize,
            mcp_is_enabled,
            mcp_get_config,
            mcp_update_config,
            mcp_list_tools,
            mcp_call_tool,
            mcp_get_context,
            mcp_enhance_transcript,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
