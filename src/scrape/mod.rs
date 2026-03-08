pub mod cidrap;
pub mod poultrysite;
pub mod poultryworld;
pub mod wattagnet;

use crate::article::Article;
use chrono::DateTime;
use reqwest;
use rss::Channel;
use scraper::{Html, Selector};
use std::error::Error;

pub async fn fetch_rss(url: &str) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    let body = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&body[..])?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut articles: Vec<Article> = Vec::new();

    for item in channel.items() {
        let raw_date = item.pub_date().unwrap_or("");
        let date_pub = normalize_rss_date(raw_date);
        if date_pub != today {
            continue;
        }
        let article_body: String = fetch_article_body(&item.link().unwrap_or(""))
            .await
            .unwrap_or_default();
        articles.push(Article {
            title: item.title().unwrap_or("No title found").to_string(),
            link: item.link().unwrap_or("No link found").to_string(),
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

async fn fetch_article_body(url: &str) -> Result<String, Box<dyn Error>> {
    let html = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&html);
    let selector = Selector::parse("p").unwrap();
    let body_text: String = document
        .select(&selector)
        .map(|el| el.text().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n");
    Ok(body_text)
}
