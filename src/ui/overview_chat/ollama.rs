// ...existing code...

// ...existing code...

pub const OLLAMA_URL: &str = "http://127.0.0.1:11434";

#[derive(Clone, Debug)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Default)]
pub struct OllamaChatOptions {
    pub num_predict: Option<i32>,
    pub temperature: Option<f64>,
    pub seed: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct ParsedAssistant {
    pub content: String,
    pub prompt_eval_count: Option<u64>,
    pub eval_count: Option<u64>,
}
