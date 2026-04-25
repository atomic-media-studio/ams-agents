use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use super::{OLLAMA_STOPPED_MSG, OllamaStopEpoch, TokenUsage, client};

#[derive(Clone)]
pub(crate) struct InferenceEngine {
    host: String,
    client: reqwest::Client,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct InferenceOptions {
    pub max_output_tokens: Option<u32>,
    pub context_window: Option<u32>,
}

pub(crate) struct InferenceRequest<'a> {
    pub instruction: &'a str,
    pub input: &'a str,
    pub model_override: Option<&'a str>,
    pub options: InferenceOptions,
    pub stop_epoch: Option<OllamaStopEpoch>,
}

pub(crate) struct InferenceResponse {
    pub model: String,
    pub text: String,
    pub usage: Option<TokenUsage>,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    stream: bool,
    messages: Vec<ChatMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<ChatOptions>,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct ChatOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<u32>,
}

#[derive(Deserialize)]
struct ChatResponse {
    model: Option<String>,
    message: Option<ChatMessageOwned>,
    response: Option<String>,
    prompt_eval_count: Option<u64>,
    eval_count: Option<u64>,
}

#[derive(Deserialize)]
struct ChatMessageOwned {
    content: String,
}

impl InferenceEngine {
    pub(crate) fn from_host(ollama_host: &str) -> Arc<Self> {
        static CACHE: OnceLock<Mutex<HashMap<String, Arc<InferenceEngine>>>> = OnceLock::new();
        let host = client::normalize_ollama_host(ollama_host);
        let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        let mut guard = cache.lock().unwrap();
        if let Some(existing) = guard.get(&host) {
            return existing.clone();
        }
        let engine = Arc::new(Self {
            host: host.clone(),
            client: reqwest::Client::new(),
        });
        guard.insert(host, engine.clone());
        engine
    }

    pub(crate) async fn infer(&self, req: InferenceRequest<'_>) -> Result<InferenceResponse> {
        if let Some((epoch, caught)) = &req.stop_epoch
            && epoch.load(std::sync::atomic::Ordering::SeqCst) != *caught
        {
            return Err(anyhow!(OLLAMA_STOPPED_MSG));
        }

        let model = if let Some(m) = req.model_override {
            if m.trim().is_empty() {
                std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "glm-4.7-flash:latest".to_string())
            } else {
                m.to_string()
            }
        } else {
            std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "glm-4.7-flash:latest".to_string())
        };

        let url = format!("{}/api/chat", self.host);
        crate::web::guard_ollama_request(&url)?;

        let options = if req.options.max_output_tokens.is_some() || req.options.context_window.is_some() {
            Some(ChatOptions {
                num_predict: req.options.max_output_tokens,
                num_ctx: req.options.context_window,
            })
        } else {
            None
        };

        let payload = ChatRequest {
            model: &model,
            stream: false,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: req.instruction,
                },
                ChatMessage {
                    role: "user",
                    content: req.input,
                },
            ],
            options,
        };

        let response = self
            .client
            .post(url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        if let Some((epoch, caught)) = &req.stop_epoch
            && epoch.load(std::sync::atomic::Ordering::SeqCst) != *caught
        {
            return Err(anyhow!(OLLAMA_STOPPED_MSG));
        }

        let parsed: ChatResponse = response.json().await?;
        let text = if let Some(message) = parsed.message {
            message.content
        } else if let Some(text) = parsed.response {
            text
        } else {
            return Err(anyhow!("ollama returned an empty response"));
        };
        let usage = match (parsed.prompt_eval_count, parsed.eval_count) {
            (Some(prompt), Some(candidates)) => Some(TokenUsage {
                prompt_token_count: prompt,
                candidates_token_count: candidates,
                total_token_count: prompt + candidates,
            }),
            _ => None,
        };

        Ok(InferenceResponse {
            model: parsed.model.unwrap_or(model),
            text,
            usage,
        })
    }
}
