use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== CORE TYPES =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingHistoryItem {
    pub id: String,
    pub created_at: i64,
    pub duration: f64,
    pub transcript: String,
    pub original_transcript: Option<String>,
    pub is_original_shown: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppRule {
    pub id: String,
    pub app_name: String,
    pub executable: Option<String>,
    pub enabled: bool,
    pub shortcut: Option<String>,
    pub hold_key: Option<String>,
    pub key_combination: Option<String>,
    pub stt_provider_id: Option<String>,
    pub transcript_post_processing_enabled: Option<bool>,
    pub transcript_post_processing_provider_id: Option<String>,
    pub transcript_post_processing_prompt: Option<String>,
    pub auto_insert: Option<bool>,
    pub priority: i32,
    pub context_formatting: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsProfile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceActivationStatus {
    pub is_enabled: bool,
    pub is_listening: bool,
    pub audio_level: f32,
    pub recording_start_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingDictationStatus {
    pub is_enabled: bool,
    pub is_active: bool,
    pub is_listening: bool,
    pub current_text: String,
    pub last_final_text: String,
    pub confidence: f32,
    pub audio_level: f32,
    pub language: String,
    pub start_time: i64,
    pub words_spoken: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveApplication {
    pub name: String,
    pub executable: String,
    pub title: String,
    pub last_updated: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesData {
    pub profiles: Vec<SettingsProfile>,
    pub active_profile_id: Option<String>,
}
