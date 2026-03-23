pub mod cidrap;
pub mod poultrysite;
pub mod poultryworld;
pub mod wattagnet;

use crate::article::Article;
use crate::error::AppError;
use chrono::DateTime;
use rss::Channel;
use scraper::{Html, Selector};
use url::Url;

pub async fn fetch_rss(url: &str) -> Result<Vec<Article>, AppError> {
    let body = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&body[..])?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut articles: Vec<Article> = Vec::new();

    for item in channel.items() {
        let link = match item.link() {
            Some(l) if is_safe_url(l) => l,
            _ => continue,
        };
        let raw_date = item.pub_date().unwrap_or("");
        let date_pub = normalize_rss_date(raw_date);
        if date_pub != today {
            continue;
        }

        let article_body = match fetch_article_body(link).await {
            Ok(body) => body,
            Err(e) => {
                eprintln!("Failed to fetch body for {}: {}", link, e);
                String::new()
            }
        };

        articles.push(Article {
            title: item.title().unwrap_or("No title found").to_string(),
            link: link.to_string(),
            summary: item.description().unwrap_or("No summary found").to_string(),
            body: Some(article_body),
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

async fn fetch_article_body(url: &str) -> Result<String, AppError> {
    let html = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&html);

    let selector = Selector::parse("p")
        .map_err(|e| AppError::HtmlParse(format!("Invalid CSS selector 'p': {}", e)))?;

    let body_text: String = document
        .select(&selector)
        .map(|el| el.text().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n");

    Ok(body_text)
}

pub fn is_safe_url(url: &str) -> bool {
    match Url::parse(url) {
        Ok(parsed) => matches!(parsed.scheme(), "http" | "https"),
        Err(_) => false,
    }
}
