use crate::article::Article;
use crate::scrape;
use crate::error::AppError;

pub async fn fetch(url: &str) -> Result<Vec<Article>, AppError> {
    scrape::fetch_rss(url).await
}
