use crate::article::Article;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::time::Duration;

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

pub async fn summarize_articles(articles: &[Article]) -> Result<String, Box<dyn Error>> {
    let api_key = env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY not found in env vars".to_string())?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let formatted_articles = articles
        .iter()
        .filter_map(|item| item.body.as_deref())
        .collect::<Vec<&str>>()
        .join("\n");

    let prompt = format!(
        "Summarize these articles for a weekly news letter about bird flu:\n{}",
        formatted_articles
    );

    let request_body = AnthropicRequest {
        model: "claude-sonnet-4-6".to_string(),
        max_tokens: 1024,
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(format!("Anthropic API error ({}): {}", status, body).into());
    }

    let parsed: AnthropicResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse Anthropic response: {}\nBody: {}", e, body))?;

    Ok(parsed.content[0].text.clone())
}
