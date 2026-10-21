// tauri-bridge.js
// Universal bridge mapping window.electronAPI and window.api to Tauri v2 Rust commands and events

(function () {
  if (window.electronAPI) {
    // Already provided (e.g. running in Electron)
    return;
  }

  const tauri = window.__TAURI__;
  if (!tauri) {
    console.warn("Tauri API not found on window.__TAURI__. Make sure withGlobalTauri is enabled in tauri.conf.json");
    return;
  }

  const { invoke } = tauri.core;
  const { listen } = tauri.event;

  // Active event unlisteners
  const listeners = new Map();

  function addEventListener(channel, callback) {
    let unlistenPromise = listen(channel, (event) => {
      // Emulate Electron signature (_event, data)
      try {
        callback(event, event.payload);
      } catch (err) {
        console.error(`Error in event handler for "${channel}":`, err);
      }
    });
    if (!listeners.has(channel)) {
      listeners.set(channel, []);
    }
    listeners.get(channel).push(unlistenPromise);
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }

  window.electronAPI = {
    // Screenshot and OCR
    takeScreenshot: () => invoke("take_screenshot"),
    captureArea: (options) => invoke("capture_area", { options }),
    listDisplays: () => invoke("list_displays"),

    // Speech recognition
    startSpeechRecognition: () => invoke("start_speech_recognition"),
    stopSpeechRecognition: () => invoke("stop_speech_recognition"),
    sendAudioChunk: (buffer) => invoke("send_audio_chunk", { buffer }),
    getSpeechAvailability: () => invoke("get_speech_availability"),

    // Window management
    showAllWindows: () => invoke("show_all_windows"),
    hideAllWindows: () => invoke("hide_all_windows"),
    enableWindowInteraction: () => invoke("enable_window_interaction"),
    disableWindowInteraction: () => invoke("disable_window_interaction"),
    switchToChat: () => invoke("switch_to_chat"),
    switchToSkills: () => invoke("switch_to_skills"),
    resizeWindow: (width, height) => invoke("resize_window", { width, height }),
    moveWindow: (deltaX, deltaY) => invoke("move_window", { deltaX, deltaY }),
    getWindowStats: () => invoke("get_window_stats"),
    closeWindow: () => invoke("close_window"),
    notifyMainWindowReady: () => invoke("notify_main_window_ready"),
    quit: () => invoke("quit_app"),

    // Session memory
    getSessionHistory: () => invoke("get_session_history"),
    getLLMSessionHistory: () => invoke("get_llm_session_history"),
    clearSessionMemory: () => invoke("clear_session_memory"),
    formatSessionHistory: () => invoke("format_session_history"),
    sendChatMessage: (text) => invoke("send_chat_message", { text }),
    getSkillPrompt: (skillName) => invoke("get_skill_prompt", { skillName }),

    // LLM configuration & status
    setGeminiApiKey: (apiKey) => invoke("set_gemini_api_key", { apiKey }),
    getGeminiStatus: () => invoke("get_gemini_status"),
    testGeminiConnection: () => invoke("test_gemini_connection"),

    // Settings
    showSettings: () => invoke("show_settings"),
    hideSettings: () => invoke("hide_settings"),
    getSettings: () => invoke("get_settings"),
    saveSettings: (settings) => invoke("save_settings", { settings }),

    // First-run onboarding
    getFirstRunStatus: () => invoke("get_first_run_status"),
    completeFirstRun: () => invoke("complete_first_run"),
    openExternal: (url) => invoke("open_external", { url }),
    closeOnboarding: () => invoke("close_onboarding"),
    detectWhisper: () => invoke("detect_whisper"),
    installWhisper: () => invoke("install_whisper"),
    downloadWhisperModel: (modelName) => invoke("download_whisper_model", { modelName }),

    // UI helpers
    updateAppIcon: (iconKey) => invoke("update_app_icon", { iconKey }),
    updateActiveSkill: (skill) => invoke("update_active_skill", { skill }),
    restartAppForStealth: () => invoke("restart_app_for_stealth"),
    expandLlmWindow: (contentMetrics) => invoke("expand_llm_window", { contentMetrics }),
    resizeLlmWindowForContent: (contentMetrics) => invoke("resize_llm_window_for_content", { contentMetrics }),
    copyToClipboard: (text) => invoke("copy_to_clipboard", { text: String(text ?? "") }),

    // Events
    onTranscriptionReceived: (cb) => addEventListener("transcription-received", cb),
    onInterimTranscription: (cb) => addEventListener("interim-transcription", cb),
    onSpeechStatus: (cb) => addEventListener("speech-status", cb),
    onSpeechError: (cb) => addEventListener("speech-error", cb),
    onSpeechAvailability: (cb) => addEventListener("speech-availability", cb),
    onSessionEvent: (cb) => addEventListener("session-event", cb),
    onSessionCleared: (cb) => addEventListener("session-cleared", cb),
    onOcrCompleted: (cb) => addEventListener("ocr-completed", cb),
    onOcrError: (cb) => addEventListener("ocr-error", cb),
    onLlmResponse: (cb) => addEventListener("llm-response", cb),
    onLlmError: (cb) => addEventListener("llm-error", cb),
    onTranscriptionLlmResponse: (cb) => addEventListener("transcription-llm-response", cb),
    onTranscriptionLlmResponseStart: (cb) => addEventListener("transcription-llm-response-start", cb),
    onTranscriptionLlmResponseChunk: (cb) => addEventListener("transcription-llm-response-chunk", cb),
    onOpenGeminiConfig: (cb) => addEventListener("open-gemini-config", cb),
    onDisplayLlmResponse: (cb) => addEventListener("display-llm-response", cb),
    onShowLoading: (cb) => addEventListener("show-loading", cb),
    onSkillChanged: (cb) => addEventListener("skill-changed", cb),
    onInteractionModeChanged: (cb) => addEventListener("interaction-mode-changed", cb),
    onRecordingStarted: (cb) => addEventListener("recording-started", cb),
    onRecordingStopped: (cb) => addEventListener("recording-stopped", cb),
    onCodingLanguageChanged: (cb) => addEventListener("coding-language-changed", cb),
    onMainWindowShown: (cb) => addEventListener("main-window-shown", cb),
    onInstallProgress: (cb) => addEventListener("install-progress", cb),

    receive: (channel, cb) => addEventListener(channel, cb),
    removeAllListeners: (channel) => {
      const channelListeners = listeners.get(channel) || [];
      channelListeners.forEach((p) => p.then((unlisten) => unlisten()));
      listeners.delete(channel);
    }
  };

  // Legacy window.api bridge
  window.api = {
    send: (channel, data) => {
      invoke("legacy_api_send", { channel, data: data ? JSON.stringify(data) : null });
    },
    receive: (channel, func) => addEventListener(channel, func)
  };
})();
