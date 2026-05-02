use crate::article::Article;
use crate::error::AppError;
use crate::scrape;

pub async fn fetch(url: &str) -> Result<Vec<Article>, AppError> {
    let client = scrape::build_client()?;
    let response = client.get(url).send().await?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::Config(format!("HTTP {} from {}", status, url)));
    }

    let body = response.bytes().await?;
    let channel = rss::Channel::read_from(&body[..])?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut articles: Vec<Article> = Vec::new();

    for item in channel.items() {
        let google_link = match item.link() {
            Some(l) => l,
            None => continue,
        };

        let real_link = match resolve_redirect(&client, google_link).await {
            Ok(u) => u,
            Err(e) => {
                eprintln!(
                    "WattAgNet: failed to resolve redirect for {}: {}",
                    google_link, e
                );
                continue;
            }
        };

        if !real_link.contains("wattagnet.com") {
            continue;
        }

        if !scrape::is_safe_url(&real_link) {
            continue;
        }

        let raw_date = item.pub_date().unwrap_or("");
        let date_pub = scrape::normalize_rss_date(raw_date);
        if date_pub != today {
            continue;
        }

        let article_body = scrape::fetch_article_body(&real_link)
            .await
            .unwrap_or_default();

        articles.push(Article {
            title: item.title().unwrap_or("No title found").to_string(),
            link: real_link,
            summary: item.description().unwrap_or("No summary found").to_string(),
            body: Some(article_body),
            date_pub,
            source: "WattAgNet (via Google News)".to_string(),
            fetched_at: chrono::offset::Local::now().to_rfc3339(),
        });
    }

    Ok(articles)
}

async fn resolve_redirect(client: &reqwest::Client, google_url: &str) -> Result<String, AppError> {
    let resp = client
        .head(google_url)
        .send()
        .await
        .map_err(|e| AppError::Config(format!("redirect resolve failed: {}", e)))?;

    Ok(resp.url().to_string())
}
