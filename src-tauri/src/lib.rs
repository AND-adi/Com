mod commands;
mod config;
mod llm;
mod stealth;

use config::ConfigState;
use llm::LLMService;
use stealth::apply_stealth_to_window;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                let _ = app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                );
            }

            let app_data_dir = app.path().app_data_dir().ok();
            app.manage(ConfigState::new(app_data_dir));
            app.manage(LLMService::new());
            app.manage(commands::SessionState::new());

            // Apply stealth protection to all initialized windows
            for (_, win) in app.webview_windows() {
                apply_stealth_to_window(&win);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::show_all_windows,
            commands::hide_all_windows,
            commands::enable_window_interaction,
            commands::disable_window_interaction,
            commands::switch_to_chat,
            commands::switch_to_skills,
            commands::resize_window,
            commands::move_window,
            commands::get_window_stats,
            commands::close_window,
            commands::notify_main_window_ready,
            commands::quit_app,
            commands::take_screenshot,
            commands::capture_area,
            commands::list_displays,
            commands::start_speech_recognition,
            commands::stop_speech_recognition,
            commands::send_audio_chunk,
            commands::get_speech_availability,
            commands::send_chat_message,
            commands::get_session_history,
            commands::get_llm_session_history,
            commands::clear_session_memory,
            commands::format_session_history,
            commands::get_skill_prompt,
            commands::set_gemini_api_key,
            commands::get_gemini_status,
            commands::test_gemini_connection,
            commands::show_settings,
            commands::hide_settings,
            commands::get_settings,
            commands::save_settings,
            commands::get_first_run_status,
            commands::complete_first_run,
            commands::open_external,
            commands::close_onboarding,
            commands::detect_whisper,
            commands::install_whisper,
            commands::download_whisper_model,
            commands::update_app_icon,
            commands::update_active_skill,
            commands::restart_app_for_stealth,
            commands::expand_llm_window,
            commands::resize_llm_window_for_content,
            commands::copy_to_clipboard,
            commands::legacy_api_send,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenCluely tauri application");
}
