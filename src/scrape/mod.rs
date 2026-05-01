pub mod cidrap;
pub mod poultrysite;
pub mod poultryworld;
pub mod wattagnet;

use crate::article::Article;
use crate::error::AppError;
use chrono::DateTime;
use reqwest;
use rss::Channel;
use scraper::{Html, Selector};
use url::Url;

fn build_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:150.0) Gecko/20100101 Firefox/150.0",
        )
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::ACCEPT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                    .parse()
                    .unwrap(),
            );
            headers.insert(
                reqwest::header::ACCEPT_LANGUAGE,
                "en-US,en;q=0.9".parse().unwrap(),
            );
            headers.insert(
                reqwest::header::ACCEPT_ENCODING,
                "gzip, deflate, br".parse().unwrap(),
            );
            headers.insert(reqwest::header::CONNECTION, "keep-alive".parse().unwrap());
            headers
        })
        .build()
}

pub async fn fetch_rss(url: &str) -> Result<Vec<Article>, AppError> {
    let client = build_client()?;
    let response = client.get(url).send().await?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::Config(format!("HTTP {} from {}", status, url)));
    }

    let body = response.bytes().await?;
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
        let article_body: String = fetch_article_body(&item.link().unwrap_or(""))
            .await
            .unwrap_or_default();
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

pub async fn fetch_rss_curl(url: &str) -> Result<Vec<Article>, AppError> {
    let output = tokio::process::Command::new("curl")
        .args([
            "-s",             // silent, no progress bar
            "-f",             // fail fast on HTTP errors (returns exit code 22)
            "-L",             // follow redirects
            "--max-time", "15",
            "-H", "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:150.0) Gecko/20100101 Firefox/150.0",
            "-H", "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            "-H", "Accept-Language: en-US,en;q=0.9",
            "--compressed",   // let curl handle gzip/br/deflate decoding
            url,
        ])
        .output()
        .await
        .map_err(|e| AppError::Config(format!("Failed to run curl: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Config(format!(
            "curl failed for {} (exit {}): {}",
            url,
            output.status.code().unwrap_or(-1),
            stderr.chars().take(200).collect::<String>()
        )));
    }

    // curl --compressed already decodes gzip/br, so output.stdout is raw text.
    // Try UTF-8 first; fall back to Latin-1 if the feed has stray non-UTF-8 bytes.
    let body: Vec<u8> = match std::str::from_utf8(&output.stdout) {
        Ok(_) => output.stdout,
        Err(_) => {
            let text: String = output.stdout.iter().map(|&b| b as char).collect();
            text.into_bytes()
        }
    };

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
        let article_body: String = fetch_article_body(link).await.unwrap_or_default();
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

pub async fn fetch_article_body(url: &str) -> Result<String, AppError> {
    let client = build_client()?;
    let html = client.get(url).send().await?.text().await?;
    let document = Html::parse_document(&html);
    let selector = Selector::parse("p").map_err(|e| AppError::HtmlParse(format!("{}", e)))?;
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
