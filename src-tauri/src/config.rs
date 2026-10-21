use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub gemini_key: Option<String>,
    #[serde(default)]
    pub speech_provider: Option<String>,
    #[serde(default)]
    pub azure_key: Option<String>,
    #[serde(default)]
    pub azure_region: Option<String>,
    #[serde(default)]
    pub whisper_command: Option<String>,
    #[serde(default)]
    pub whisper_model: Option<String>,
    #[serde(default)]
    pub whisper_language: Option<String>,
    #[serde(default)]
    pub whisper_device: Option<String>,
    #[serde(default)]
    pub whisper_capture_mode: Option<String>,
    #[serde(default)]
    pub whisper_response_target: Option<String>,
    #[serde(default)]
    pub whisper_segment_ms: Option<u32>,
    #[serde(default)]
    pub window_gap: Option<u32>,
    #[serde(default)]
    pub coding_language: Option<String>,
    #[serde(default)]
    pub active_skill: Option<String>,
    #[serde(default)]
    pub stealth_mode: Option<bool>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            gemini_key: None,
            speech_provider: Some("whisper".to_string()),
            azure_key: None,
            azure_region: None,
            whisper_command: Some("whisper".to_string()),
            whisper_model: Some("base".to_string()),
            whisper_language: Some("en".to_string()),
            whisper_device: Some("auto".to_string()),
            whisper_capture_mode: Some("vad".to_string()),
            whisper_response_target: Some("both".to_string()),
            whisper_segment_ms: Some(4000),
            window_gap: Some(10),
            coding_language: Some("cpp".to_string()),
            active_skill: Some("general".to_string()),
            stealth_mode: Some(true),
        }
    }
}

pub struct ConfigState {
    pub settings: Mutex<AppSettings>,
    pub env_path: PathBuf,
}

impl ConfigState {
    pub fn new(app_data_dir: Option<PathBuf>) -> Self {
        let env_path = if let Some(dir) = app_data_dir {
            dir.join(".env")
        } else {
            PathBuf::from(".env")
        };

        let mut settings = AppSettings::default();

        if env_path.exists() {
            let _ = dotenvy::from_path(&env_path);
        } else if Path::new(".env").exists() {
            let _ = dotenvy::from_path(".env");
        }

        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            settings.gemini_key = Some(key);
        }
        if let Ok(provider) = std::env::var("SPEECH_PROVIDER") {
            settings.speech_provider = Some(provider);
        }
        if let Ok(model) = std::env::var("WHISPER_MODEL") {
            settings.whisper_model = Some(model);
        }
        if let Ok(lang) = std::env::var("WHISPER_LANGUAGE") {
            settings.whisper_language = Some(lang);
        }
        if let Ok(dev) = std::env::var("WHISPER_DEVICE") {
            settings.whisper_device = Some(dev);
        }
        if let Ok(target) = std::env::var("WHISPER_RESPONSE_TARGET") {
            settings.whisper_response_target = Some(target);
        }

        Self {
            settings: Mutex::new(settings),
            env_path,
        }
    }

    pub fn get_settings(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn save(&self, new_settings: AppSettings) -> Result<(), String> {
        let mut settings = self.settings.lock().map_err(|e| e.to_string())?;

        // Merge updates
        if new_settings.gemini_key.is_some() {
            settings.gemini_key = new_settings.gemini_key;
        }
        if new_settings.speech_provider.is_some() {
            settings.speech_provider = new_settings.speech_provider;
        }
        if new_settings.whisper_model.is_some() {
            settings.whisper_model = new_settings.whisper_model;
        }
        if new_settings.whisper_language.is_some() {
            settings.whisper_language = new_settings.whisper_language;
        }
        if new_settings.whisper_device.is_some() {
            settings.whisper_device = new_settings.whisper_device;
        }
        if new_settings.coding_language.is_some() {
            settings.coding_language = new_settings.coding_language;
        }
        if new_settings.active_skill.is_some() {
            settings.active_skill = new_settings.active_skill;
        }
        if new_settings.window_gap.is_some() {
            settings.window_gap = new_settings.window_gap;
        }

        // Persist to .env
        let mut content = String::new();
        if let Some(ref key) = settings.gemini_key {
            content.push_str(&format!("GEMINI_API_KEY={}\n", key));
        }
        if let Some(ref sp) = settings.speech_provider {
            content.push_str(&format!("SPEECH_PROVIDER={}\n", sp));
        }
        if let Some(ref model) = settings.whisper_model {
            content.push_str(&format!("WHISPER_MODEL={}\n", model));
        }
        if let Some(ref lang) = settings.whisper_language {
            content.push_str(&format!("WHISPER_LANGUAGE={}\n", lang));
        }
        if let Some(ref dev) = settings.whisper_device {
            content.push_str(&format!("WHISPER_DEVICE={}\n", dev));
        }
        if let Some(ref target) = settings.whisper_response_target {
            content.push_str(&format!("WHISPER_RESPONSE_TARGET={}\n", target));
        }

        let _ = fs::write(&self.env_path, content);
        Ok(())
    }
}
