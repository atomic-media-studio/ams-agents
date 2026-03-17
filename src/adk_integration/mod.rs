use anyhow::Result;

mod ollama;

pub async fn fetch_ollama_models() -> Result<Vec<String>> {
    ollama::fetch_models().await
}

pub async fn send_to_ollama(
    instruction: &str,
    input: &str,
    limit_token: bool,
    num_predict: &str,
    model_override: Option<&str>,
) -> Result<String> {
    send_to_ollama_with_context(instruction, input, limit_token, num_predict, model_override).await
}

pub async fn send_to_ollama_with_context(
    instruction: &str,
    input: &str,
    limit_token: bool,
    num_predict: &str,
    model_override: Option<&str>,
) -> Result<String> {
    let runner_ctx =
        ollama::build_runner_context(instruction, limit_token, num_predict, model_override).await?;
    ollama::print_context_preview(input);
    ollama::run_prompt_streaming(runner_ctx, input, false).await
}

pub async fn test_ollama(model_override: Option<&str>) -> Result<String> {
    let runner_ctx = ollama::build_runner_context(
        "You are a helpful assistant running locally via Ollama.",
        false,
        "",
        model_override,
    )
    .await?;
    let input = "Hello, how are you?";
    println!("Input: {}", input);
    ollama::run_prompt_streaming(runner_ctx, input, true).await
}

