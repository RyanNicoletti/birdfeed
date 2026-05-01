use crate::article::Article;
use crate::error::AppError;
use crate::scrape;

pub async fn fetch(url: &str) -> Result<Vec<Article>, AppError> {
    scrape::fetch_rss_curl(url).await
}
