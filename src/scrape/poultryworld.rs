use crate::article::Article;
use chrono::Utc;
use scraper::{Html, Selector};

pub async fn fetch_from_pw(url: &str) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    let html = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&html);

    let mut articles: Vec<Article> = Vec::new();

    let text_grid_selector = Selector::parse(".text-grid").unwrap();
    let h3_link_selector = Selector::parse("h3 a").unwrap();
    let time_selector = Selector::parse(".meta-t .time").unwrap();

    let current_year = Utc::now().format("%Y").to_string();

    for element in document.select(&text_grid_selector) {
        let link_element = element.select(&h3_link_selector).next();

        if let Some(link_el) = link_element {
            let link = link_el.value().attr("href").unwrap_or("").to_string();
            let title = link_el.text().collect::<String>().trim().to_string();
            let raw_date = element
                .select(&time_selector)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let date_pub = normalize_date(&raw_date, &current_year);

            if !title.is_empty() && !link.is_empty() {
                articles.push(Article {
                    title,
                    link,
                    summary: String::new(),
                    date_pub,
                    source: url.to_owned(),
                });
            }
        }
    }

    Ok(articles)
}

/// Converts dates like "26-01" or "28-12-2025" to "YYYY-MM-DD" format
fn normalize_date(raw_date: &str, current_year: &str) -> String {
    let parts: Vec<&str> = raw_date.split('-').collect();

    match parts.len() {
        2 => {
            // "26-01" -> day-month, assume current year
            let day = parts[0];
            let month = parts[1];
            format!("{}-{}-{}", current_year, month, day)
        }
        3 => {
            // "28-12-2025" -> day-month-year
            let day = parts[0];
            let month = parts[1];
            let year = parts[2];
            format!("{}-{}-{}", year, month, day)
        }
        _ => raw_date.to_string(),
    }
}
