use crate::types::{ProfilesData, SettingsProfile};
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ConfigStore {
    config_path: PathBuf,
    profiles_path: PathBuf,
    current_config: Mutex<Value>,
    profiles_data: Mutex<ProfilesData>,
}

impl ConfigStore {
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&app_data_dir).context("Failed to create app data directory")?;

        let config_path = app_data_dir.join("config.json");
        let profiles_path = app_data_dir.join("profiles.json");

        // Load or create config
        let current_config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| default_config())
        } else {
            default_config()
        };

        // Load or create profiles
        let profiles_data = if profiles_path.exists() {
            let content = fs::read_to_string(&profiles_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| ProfilesData {
                profiles: vec![],
                active_profile_id: None,
            })
        } else {
            ProfilesData {
                profiles: vec![],
                active_profile_id: None,
            }
        };

        Ok(Self {
            config_path,
            profiles_path,
            current_config: Mutex::new(current_config),
            profiles_data: Mutex::new(profiles_data),
        })
    }

    pub fn get(&self) -> Value {
        self.current_config.lock().unwrap().clone()
    }

    pub fn save(&self, config: Value) -> Result<()> {
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&self.config_path, content)?;
        *self.current_config.lock().unwrap() = config;
        Ok(())
    }

    // ===== PROFILES MANAGEMENT =====

    pub fn get_profiles(&self) -> Vec<SettingsProfile> {
        self.profiles_data.lock().unwrap().profiles.clone()
    }

    pub fn get_active_profile_id(&self) -> Option<String> {
        self.profiles_data.lock().unwrap().active_profile_id.clone()
    }

    pub fn create_profile(
        &self,
        name: String,
        description: Option<String>,
        base_config: Option<Value>,
    ) -> Result<SettingsProfile> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let profile = SettingsProfile {
            id: format!("profile_{}", timestamp),
            name,
            description,
            config: base_config.unwrap_or_else(default_config),
            created_at: timestamp,
            updated_at: timestamp,
            is_default: None,
        };

        let mut data = self.profiles_data.lock().unwrap();
        data.profiles.push(profile.clone());
        drop(data);

        self.save_profiles()?;

        Ok(profile)
    }

    pub fn update_profile(&self, profile_id: String, updates: Value) -> Result<()> {
        let mut data = self.profiles_data.lock().unwrap();

        if let Some(profile) = data.profiles.iter_mut().find(|p| p.id == profile_id) {
            // Merge updates into profile
            if let Some(name) = updates.get("name").and_then(|v| v.as_str()) {
                profile.name = name.to_string();
            }
            if let Some(desc) = updates.get("description").and_then(|v| v.as_str()) {
                profile.description = Some(desc.to_string());
            }
            if let Some(config) = updates.get("config") {
                profile.config = config.clone();
            }

            profile.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;

            drop(data);
            self.save_profiles()?;
            Ok(())
        } else {
            anyhow::bail!("Profile not found")
        }
    }

    pub fn delete_profile(&self, profile_id: String) -> Result<bool> {
        let mut data = self.profiles_data.lock().unwrap();

        let initial_len = data.profiles.len();
        data.profiles.retain(|p| p.id != profile_id);

        if data.profiles.len() < initial_len {
            // If deleted profile was active, clear active profile
            if data.active_profile_id.as_ref() == Some(&profile_id) {
                data.active_profile_id = None;
            }

            drop(data);
            self.save_profiles()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn switch_profile(&self, profile_id: String) -> Result<bool> {
        let mut data = self.profiles_data.lock().unwrap();

        if let Some(profile) = data.profiles.iter().find(|p| p.id == profile_id) {
            data.active_profile_id = Some(profile_id.clone());
            let config = profile.config.clone();
            drop(data);

            self.save_profiles()?;
            self.save(config)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn save_profiles(&self) -> Result<()> {
        let data = self.profiles_data.lock().unwrap();
        let content = serde_json::to_string_pretty(&*data)?;
        fs::write(&self.profiles_path, content)?;
        Ok(())
    }
}

fn default_config() -> Value {
    serde_json::json!({
        "sttProviderId": "openai",
        "openaiApiKey": "",
        "groqApiKey": "",
        "geminiApiKey": "",
        "openaiBaseUrl": "https://api.openai.com/v1",
        "groqBaseUrl": "https://api.groq.com/openai/v1",
        "geminiBaseUrl": "https://generativelanguage.googleapis.com",
        "recordingShortcut": "Control",
        "shortcut": "hold-key",
        "holdKey": "Control",
        "transcriptPostProcessingEnabled": false,
        "transcriptPostProcessingProviderId": "openai",
        "transcriptPostProcessingPrompt": "Fix any grammar or spelling errors in the following text, but maintain the original meaning and tone:",
        "appRules": [],
        "enableAppRules": false,
        "voiceActivation": {
            "enabled": false,
            "sensitivity": 50,
            "silenceThreshold": 1500,
            "noiseGate": 30,
            "minRecordingDuration": 500,
            "maxRecordingDuration": 60000
        },
        "streamingDictation": {
            "enabled": false,
            "language": "en-US",
            "continuous": true,
            "interimResults": true,
            "maxAlternatives": 1,
            "sensitivity": 50,
            "punctuationMode": "auto",
            "capitalizationMode": "auto",
            "pauseOnSilence": 2000,
            "insertMode": "insert",
            "enableVoiceCommands": false,
            "contextFormatting": false
        },
        "fusionTranscription": {
            "enabled": false,
            "strategy": "best-confidence",
            "providers": ["openai", "groq"],
            "timeoutMs": 30000,
            "minProvidersRequired": 1,
            "confidenceThreshold": 0.7,
            "enableParallel": true,
            "providerWeights": {
                "openai": 1.0,
                "groq": 1.0
            }
        },
        "contextFormatting": {
            "enabled": false,
            "autoDetectContext": true,
            "fallbackContext": "generic",
            "enableSmartFormatting": true,
            "preserveOriginalOnError": true
        }
    })
}
