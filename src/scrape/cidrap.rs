use crate::article::Article;
use crate::scrape;

pub async fn fetch(url: &str) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    scrape::fetch_rss(url).await
}
