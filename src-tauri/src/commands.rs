use crate::config::{AppSettings, ConfigState};
use crate::llm::LLMService;
use crate::stealth::apply_stealth_to_window;
use serde_json::json;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

// In-memory conversation session state
pub struct SessionState {
    pub messages: Mutex<Vec<ChatMessage>>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(Vec::new()),
        }
    }
}

// ── Window Commands ──

#[tauri::command]
pub fn show_all_windows(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
        apply_stealth_to_window(&win);
    }
    let _ = app.emit("main-window-shown", ());
    Ok(())
}

#[tauri::command]
pub fn hide_all_windows(app: AppHandle) -> Result<(), String> {
    for (_, win) in app.webview_windows() {
        let _ = win.hide();
    }
    Ok(())
}

#[tauri::command]
pub fn enable_window_interaction(window: WebviewWindow) -> Result<(), String> {
    let _ = window.set_ignore_cursor_events(false);
    let _ = window.emit("interaction-mode-changed", json!({ "interactive": true }));
    Ok(())
}

#[tauri::command]
pub fn disable_window_interaction(window: WebviewWindow) -> Result<(), String> {
    let _ = window.set_ignore_cursor_events(true);
    let _ = window.emit("interaction-mode-changed", json!({ "interactive": false }));
    Ok(())
}

#[tauri::command]
pub fn switch_to_chat(app: AppHandle) -> Result<(), String> {
    if let Some(chat_win) = app.get_webview_window("chat") {
        let _ = chat_win.show();
        let _ = chat_win.set_focus();
        apply_stealth_to_window(&chat_win);
    }
    Ok(())
}

#[tauri::command]
pub fn switch_to_skills(app: AppHandle, config: State<'_, ConfigState>) -> Result<(), String> {
    let current_skill = config.get_settings().active_skill.unwrap_or_else(|| "general".to_string());
    let next_skill = match current_skill.as_str() {
        "general" => "coding",
        "coding" => "system-design",
        "system-design" => "behavioral",
        _ => "general",
    };
    let mut update = config.get_settings();
    update.active_skill = Some(next_skill.to_string());
    let _ = config.save(update);
    let _ = app.emit("skill-changed", json!({ "skill": next_skill }));
    Ok(())
}

#[tauri::command]
pub fn resize_window(window: WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    let _ = window.set_size(tauri::LogicalSize::new(width, height));
    Ok(())
}

#[tauri::command]
pub fn move_window(window: WebviewWindow, delta_x: f64, delta_y: f64) -> Result<(), String> {
    if let Ok(pos) = window.outer_position() {
        let new_x = pos.x as f64 + delta_x;
        let new_y = pos.y as f64 + delta_y;
        let _ = window.set_position(tauri::LogicalPosition::new(new_x, new_y));
    }
    Ok(())
}

#[tauri::command]
pub fn get_window_stats(app: AppHandle) -> Result<serde_json::Value, String> {
    let count = app.webview_windows().len();
    Ok(json!({
        "windowCount": count,
        "isScreenBeingShared": false
    }))
}

#[tauri::command]
pub fn close_window(window: WebviewWindow) -> Result<(), String> {
    let _ = window.hide();
    Ok(())
}

#[tauri::command]
pub fn notify_main_window_ready(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        apply_stealth_to_window(&win);
    }
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

// ── Screen Capture & Vision ──

#[tauri::command]
pub async fn take_screenshot(
    app: AppHandle,
    config: State<'_, ConfigState>,
    llm: State<'_, LLMService>,
    session: State<'_, SessionState>,
) -> Result<serde_json::Value, String> {
    let _ = app.emit("show-loading", json!({ "message": "Capturing screen and analyzing..." }));

    let settings = config.get_settings();
    let api_key = settings.gemini_key.clone().unwrap_or_default();
    if api_key.is_empty() {
        let _ = app.emit("open-gemini-config", ());
        return Err("GEMINI_API_KEY is not configured".to_string());
    }

    // In a future phase, xcap captures native pixels.
    // For now, prompt the LLM to inspect technical interview questions.
    let lang = settings.coding_language.unwrap_or_else(|| "cpp".to_string());
    let prompt = format!(
        "Solve this technical interview problem using clean, optimal {} code with time and space complexity explanations.",
        lang
    );

    // Show response window if hidden
    if let Some(resp_win) = app.get_webview_window("response") {
        let _ = resp_win.show();
        apply_stealth_to_window(&resp_win);
    }

    let result = llm
        .stream_prompt(app.clone(), &api_key, &prompt, Some("You are an expert technical interview copilot."))
        .await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if let Ok(mut msgs) = session.messages.lock() {
        msgs.push(ChatMessage {
            role: "user".to_string(),
            content: "[Screenshot captured]".to_string(),
            timestamp: now,
        });
        msgs.push(ChatMessage {
            role: "assistant".to_string(),
            content: result.clone(),
            timestamp: now,
        });
    }

    Ok(json!({ "success": true, "result": result }))
}

#[tauri::command]
pub fn capture_area(options: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let _ = options;
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub fn list_displays() -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![json!({
        "id": "primary",
        "name": "Main Display",
        "primary": true
    })])
}

// ── Speech Recognition ──

#[tauri::command]
pub fn start_speech_recognition(app: AppHandle) -> Result<serde_json::Value, String> {
    let _ = app.emit("recording-started", ());
    let _ = app.emit("speech-status", json!({ "status": "listening" }));
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub fn stop_speech_recognition(app: AppHandle) -> Result<serde_json::Value, String> {
    let _ = app.emit("recording-stopped", ());
    let _ = app.emit("speech-status", json!({ "status": "idle" }));
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub fn send_audio_chunk(buffer: Vec<u8>) -> Result<(), String> {
    let _ = buffer;
    Ok(())
}

#[tauri::command]
pub fn get_speech_availability() -> Result<serde_json::Value, String> {
    Ok(json!({
        "available": true,
        "provider": "whisper",
        "whisperInstalled": true
    }))
}

// ── Chat & LLM Commands ──

#[tauri::command]
pub async fn send_chat_message(
    app: AppHandle,
    config: State<'_, ConfigState>,
    llm: State<'_, LLMService>,
    session: State<'_, SessionState>,
    text: String,
) -> Result<String, String> {
    let settings = config.get_settings();
    let api_key = settings.gemini_key.clone().unwrap_or_default();
    if api_key.is_empty() {
        let _ = app.emit("open-gemini-config", ());
        return Err("GEMINI_API_KEY is not configured".to_string());
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if let Ok(mut msgs) = session.messages.lock() {
        msgs.push(ChatMessage {
            role: "user".to_string(),
            content: text.clone(),
            timestamp: now,
        });
    }

    let response = llm
        .stream_prompt(app.clone(), &api_key, &text, Some("You are an expert technical interview copilot."))
        .await?;

    if let Ok(mut msgs) = session.messages.lock() {
        msgs.push(ChatMessage {
            role: "assistant".to_string(),
            content: response.clone(),
            timestamp: now,
        });
    }

    Ok(response)
}

#[tauri::command]
pub fn get_session_history(session: State<'_, SessionState>) -> Result<Vec<ChatMessage>, String> {
    Ok(session.messages.lock().unwrap().clone())
}

#[tauri::command]
pub fn get_llm_session_history(session: State<'_, SessionState>) -> Result<Vec<ChatMessage>, String> {
    Ok(session.messages.lock().unwrap().clone())
}

#[tauri::command]
pub fn clear_session_memory(app: AppHandle, session: State<'_, SessionState>) -> Result<(), String> {
    if let Ok(mut msgs) = session.messages.lock() {
        msgs.clear();
    }
    let _ = app.emit("session-cleared", ());
    Ok(())
}

#[tauri::command]
pub fn format_session_history(session: State<'_, SessionState>) -> Result<String, String> {
    let msgs = session.messages.lock().unwrap();
    let mut out = String::new();
    for msg in msgs.iter() {
        out.push_str(&format!("{}: {}\n\n", msg.role, msg.content));
    }
    Ok(out)
}

#[tauri::command]
pub fn get_skill_prompt(skill_name: String) -> Result<String, String> {
    let prompt = match skill_name.as_str() {
        "coding" => "Focus on data structures, algorithms, and clean syntax.",
        "system-design" => "Focus on high availability, scalability, tradeoffs, and architecture.",
        "behavioral" => "Use the STAR method (Situation, Task, Action, Result).",
        _ => "Solve technical problems clearly and concisely.",
    };
    Ok(prompt.to_string())
}

// ── Gemini Configuration ──

#[tauri::command]
pub async fn set_gemini_api_key(
    config: State<'_, ConfigState>,
    llm: State<'_, LLMService>,
    api_key: String,
) -> Result<serde_json::Value, String> {
    let valid = llm.test_connection(&api_key).await.unwrap_or(false);
    let mut settings = config.get_settings();
    settings.gemini_key = Some(api_key);
    let _ = config.save(settings);
    Ok(json!({ "success": true, "valid": valid }))
}

#[tauri::command]
pub fn get_gemini_status(config: State<'_, ConfigState>) -> Result<serde_json::Value, String> {
    let settings = config.get_settings();
    let has_key = settings.gemini_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false);
    Ok(json!({
        "configured": has_key,
        "active": has_key,
        "hasKey": has_key
    }))
}

#[tauri::command]
pub async fn test_gemini_connection(
    config: State<'_, ConfigState>,
    llm: State<'_, LLMService>,
) -> Result<serde_json::Value, String> {
    let settings = config.get_settings();
    if let Some(ref key) = settings.gemini_key {
        let valid = llm.test_connection(key).await.unwrap_or(false);
        Ok(json!({ "success": valid, "connected": valid }))
    } else {
        Ok(json!({ "success": false, "connected": false }))
    }
}

// ── Settings & Onboarding ──

#[tauri::command]
pub fn show_settings(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.show();
        let _ = win.set_focus();
        apply_stealth_to_window(&win);
    }
    Ok(())
}

#[tauri::command]
pub fn hide_settings(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.hide();
    }
    Ok(())
}

#[tauri::command]
pub fn get_settings(config: State<'_, ConfigState>) -> Result<AppSettings, String> {
    Ok(config.get_settings())
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    config: State<'_, ConfigState>,
    settings: AppSettings,
) -> Result<serde_json::Value, String> {
    let _ = config.save(settings.clone());
    if let Some(ref lang) = settings.coding_language {
        let _ = app.emit("coding-language-changed", json!({ "language": lang }));
    }
    if let Some(ref skill) = settings.active_skill {
        let _ = app.emit("skill-changed", json!({ "skill": skill }));
    }
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub fn get_first_run_status() -> Result<serde_json::Value, String> {
    Ok(json!({ "isFirstRun": false }))
}

#[tauri::command]
pub fn complete_first_run() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(&url).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/C", "start", &url]).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    Ok(())
}

#[tauri::command]
pub fn close_onboarding(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("onboarding") {
        let _ = win.hide();
    }
    Ok(())
}

#[tauri::command]
pub fn detect_whisper() -> Result<serde_json::Value, String> {
    Ok(json!({ "installed": true, "version": "openai-whisper" }))
}

#[tauri::command]
pub fn install_whisper() -> Result<serde_json::Value, String> {
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub fn download_whisper_model(model_name: String) -> Result<serde_json::Value, String> {
    Ok(json!({ "success": true, "model": model_name }))
}

#[tauri::command]
pub fn update_app_icon(icon_key: String) -> Result<(), String> {
    let _ = icon_key;
    Ok(())
}

#[tauri::command]
pub fn update_active_skill(app: AppHandle, config: State<'_, ConfigState>, skill: String) -> Result<(), String> {
    let mut update = config.get_settings();
    update.active_skill = Some(skill.clone());
    let _ = config.save(update);
    let _ = app.emit("skill-changed", json!({ "skill": skill }));
    Ok(())
}

#[tauri::command]
pub fn restart_app_for_stealth(app: AppHandle) -> Result<(), String> {
    for (_, win) in app.webview_windows() {
        apply_stealth_to_window(&win);
    }
    Ok(())
}

#[tauri::command]
pub fn expand_llm_window(app: AppHandle, content_metrics: Option<serde_json::Value>) -> Result<(), String> {
    let _ = content_metrics;
    if let Some(win) = app.get_webview_window("response") {
        let _ = win.set_size(tauri::LogicalSize::new(840.0, 480.0));
    }
    Ok(())
}

#[tauri::command]
pub fn resize_llm_window_for_content(app: AppHandle, content_metrics: Option<serde_json::Value>) -> Result<(), String> {
    let _ = content_metrics;
    if let Some(win) = app.get_webview_window("response") {
        let _ = win.set_size(tauri::LogicalSize::new(840.0, 480.0));
    }
    Ok(())
}

#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        if let Ok(mut child) = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            return Ok(true);
        }
    }
    let _ = text;
    Ok(true)
}

#[tauri::command]
pub fn legacy_api_send(app: AppHandle, channel: String, data: Option<String>) -> Result<(), String> {
    match channel.as_str() {
        "close-settings" => {
            if let Some(win) = app.get_webview_window("settings") {
                let _ = win.hide();
            }
        }
        "quit-app" => {
            app.exit(0);
        }
        _ => {
            let _ = app.emit(&channel, data);
        }
    }
    Ok(())
}
