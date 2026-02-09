pub mod cidrap;
pub mod poultryworld;
pub mod wattagnet;

use crate::article::Article;
use chrono::DateTime;
use rss::Channel;

pub async fn fetch_rss(url: &str) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    let body = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&body[..])?;

    let mut articles: Vec<Article> = Vec::new();

    for item in channel.items() {
        let raw_date = item.pub_date().unwrap_or("");
        let date_pub = normalize_rss_date(raw_date);

        articles.push(Article {
            title: item.title().unwrap_or("No title found").to_string(),
            link: item.link().unwrap_or("No link found").to_string(),
            summary: item.description().unwrap_or("No summary found").to_string(),
            date_pub,
            source: url.to_owned(),
            fetched_at: chrono::offset::Local::now().to_rfc3339(),
        });
    }

    Ok(articles)
}

/// convert rfc 2822 date (from rss feeds) "Mon, 09 Feb 2026 12:27:50 -0600" to "2026-02-09"
fn normalize_rss_date(raw_date: &str) -> String {
    DateTime::parse_from_rfc2822(raw_date)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|_| raw_date.to_string())
}
