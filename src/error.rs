#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("RSS parse error: {0}")]
    RssParse(#[from] rss::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("LLM API error (status {status}): {body}")]
    LlmApi { status: u16, body: String },

    #[error("LLM returned empty response")]
    LlmEmptyResponse,

    #[error("HTML parse error: {0}")]
    HtmlParse(String),
}
