use crate::types::*;
use std::sync::Mutex;

pub struct AppState {
    pub is_recording: Mutex<bool>,
    pub voice_activation: Mutex<VoiceActivationStatus>,
    pub streaming_dictation: Mutex<StreamingDictationStatus>,
    pub active_app: Mutex<Option<ActiveApplication>>,
    pub active_rule: Mutex<Option<AppRule>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            is_recording: Mutex::new(false),
            voice_activation: Mutex::new(VoiceActivationStatus {
                is_enabled: false,
                is_listening: false,
                audio_level: 0.0,
                recording_start_time: 0,
            }),
            streaming_dictation: Mutex::new(StreamingDictationStatus {
                is_enabled: false,
                is_active: false,
                is_listening: false,
                current_text: String::new(),
                last_final_text: String::new(),
                confidence: 0.0,
                audio_level: 0.0,
                language: "en-US".to_string(),
                start_time: 0,
                words_spoken: 0,
            }),
            active_app: Mutex::new(None),
            active_rule: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
