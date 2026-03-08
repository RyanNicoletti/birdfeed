use crate::article::Article;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::time::Duration;

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

pub async fn summarize_articles(articles: &[Article]) -> Result<String, Box<dyn Error>> {
    let gemini_api_key =
        env::var("GEMINI_API_KEY").map_err(|_| "Gemini key not found in env vars".to_string())?;
    let model = "gemini-3.0-flash";
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    let gemini_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, gemini_api_key
    );
    let formatted_articles = articles
        .iter()
        .filter_map(|item| item.body.as_deref())
        .collect::<Vec<&str>>()
        .join("\n");
    let prompt = format!(
        "Write a brief summary of these articles: \n{}",
        formatted_articles
    );
    let request_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part { text: prompt }],
        }],
    };
    let response = client.post(&gemini_url).json(&request_body).send().await?;
    let parsed_response: GeminiResponse = response.json().await?;
    Ok(parsed_response.candidates[0].content.parts[0]
        .text
        .to_string())
}
