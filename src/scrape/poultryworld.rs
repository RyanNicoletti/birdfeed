use crate::article::Article;
use crate::scrape::fetch_article_body;
use chrono::Utc;
use scraper::{Html, Selector};

pub async fn fetch(url: &str) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    let html = reqwest::get(url).await?.text().await?;
    let current_year = Utc::now().format("%Y").to_string();

    let items: Vec<(String, String, String)> = {
        let document = Html::parse_document(&html);
        let text_grid_selector = Selector::parse(".text-grid").unwrap();
        let h3_link_selector = Selector::parse("h3 a").unwrap();
        let time_selector = Selector::parse(".meta-t .time").unwrap();

        document
            .select(&text_grid_selector)
            .filter_map(|element| {
                let link_el = element.select(&h3_link_selector).next()?;
                let link = link_el.value().attr("href").unwrap_or("").to_string();
                let title = link_el.text().collect::<String>().trim().to_string();
                let raw_date = element
                    .select(&time_selector)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
                if title.is_empty() || link.is_empty() {
                    None
                } else {
                    Some((title, link, raw_date))
                }
            })
            .collect()
    };

    let mut articles: Vec<Article> = Vec::new();

    for (title, link, raw_date) in items {
        let date_pub = normalize_date(&raw_date, &current_year);
        let article_body = fetch_article_body(link.as_str()).await.unwrap_or_default();
        articles.push(Article {
            title,
            link,
            summary: String::new(),
            body: Some(article_body),
            date_pub,
            source: url.to_owned(),
            fetched_at: chrono::offset::Local::now().to_rfc3339(),
        });
    }

    Ok(articles)
}

/// Converts dates like "26-01" or "28-12-2025" to "YYYY-MM-DD" format
fn normalize_date(raw_date: &str, current_year: &str) -> String {
    let parts: Vec<&str> = raw_date.split('-').collect();

    match parts.len() {
        2 => {
            let day = parts[0];
            let month = parts[1];
            format!("{}-{}-{}", current_year, month, day)
        }
        3 => {
            let day = parts[0];
            let month = parts[1];
            let year = parts[2];
            format!("{}-{}-{}", year, month, day)
        }
        _ => raw_date.to_string(),
    }
}
