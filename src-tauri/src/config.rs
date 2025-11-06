use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

pub struct ConfigStore {
    config_path: PathBuf,
}

impl ConfigStore {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let config_path = app_data_dir.join("config.json");

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        Self { config_path }
    }

    pub fn get(&self) -> Value {
        if self.config_path.exists() {
            fs::read_to_string(&self.config_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_else(|| self.default_config())
        } else {
            self.default_config()
        }
    }

    pub fn save(&mut self, config: Value) -> Result<()> {
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    fn default_config(&self) -> Value {
        serde_json::json!({
            "sttProviderId": "openai",
            "openaiApiKey": "",
            "groqApiKey": "",
            "openaiBaseUrl": "https://api.openai.com/v1",
            "groqBaseUrl": "https://api.groq.com/openai/v1",
            "recordingShortcut": "Control",
            "appRules": [],
            "profiles": [],
            "activeProfileId": null,
            "fusionTranscription": {
                "enabled": false,
            },
            "voiceActivation": {
                "enabled": false,
            },
        })
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        // This will be properly initialized in the app
        Self::new(PathBuf::from("."))
    }
}
