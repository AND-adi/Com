use futures_util::StreamExt;
use reqwest::Client;
use serde_json::json;
use tauri::{AppHandle, Emitter};

pub struct LLMService {
    client: Client,
}

impl LLMService {
    pub fn new() -> Self {
        Self {
            client: Client::builder().build().unwrap_or_default(),
        }
    }

    pub async fn test_connection(&self, api_key: &str) -> Result<bool, String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
            api_key
        );

        let body = json!({
            "contents": [{
                "parts": [{ "text": "ping" }]
            }]
        });

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to reach Gemini: {}", e))?;

        if resp.status().is_success() {
            Ok(true)
        } else {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            Err(format!("Gemini API error ({}): {}", status, err_text))
        }
    }

    pub async fn stream_prompt(
        &self,
        app: AppHandle,
        api_key: &str,
        prompt: &str,
        system_instruction: Option<&str>,
    ) -> Result<String, String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:streamGenerateContent?alt=sse&key={}",
            api_key
        );

        let mut body = json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }]
        });

        if let Some(sys) = system_instruction {
            body["system_instruction"] = json!({
                "parts": [{ "text": sys }]
            });
        }

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            let err = response.text().await.unwrap_or_default();
            let _ = app.emit("llm-error", json!({ "error": err }));
            return Err(format!("Gemini request failed: {}", err));
        }

        let _ = app.emit("transcription-llm-response-start", ());
        let mut full_text = String::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk_res) = stream.next().await {
            if let Ok(chunk) = chunk_res {
                let chunk_str = String::from_utf8_lossy(&chunk);
                for line in chunk_str.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(text) = parsed["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                                full_text.push_str(text);
                                let _ = app.emit("transcription-llm-response-chunk", json!({ "text": text }));
                            }
                        }
                    }
                }
            }
        }

        let _ = app.emit("transcription-llm-response", json!({ "text": full_text }));
        let _ = app.emit("display-llm-response", json!({ "text": full_text }));
        let _ = app.emit("llm-response", json!({ "text": full_text }));

        Ok(full_text)
    }
}
