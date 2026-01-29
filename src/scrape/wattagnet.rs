use crate::article::Article;
use scraper::{Html, Selector};

pub async fn fetch_articles(url: &str) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?.text().await?;

    let document = Html::parse_document(&response);

    let article_selector = Selector::parse("div.section-feed-content-node")
        .map_err(|e| format!("Failed to parse article selector: {:?}", e))?;
    let title_selector = Selector::parse("h5.section-feed-content-node__content-short-name a")
        .map_err(|e| format!("Failed to parse title selector: {:?}", e))?;
    let teaser_selector = Selector::parse("div.section-feed-content-node__content-teaser a")
        .map_err(|e| format!("Failed to parse teaser selector: {:?}", e))?;

    let mut articles: Vec<Article> = Vec::new();

    for article_element in document.select(&article_selector) {
        let Some(title_el) = article_element.select(&title_selector).next() else {
            continue; // Skip articles without a title
        };

        let title = title_el.text().collect::<String>().trim().to_string();

        let link = title_el
            .value()
            .attr("href")
            .map(|href| {
                if href.starts_with('/') {
                    format!("https://www.wattagnet.com{}", href)
                } else {
                    href.to_string()
                }
            })
            .unwrap_or_default();

        let summary = article_element
            .select(&teaser_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "No summary available".to_string());

        let date_pub = String::new();

        articles.push(Article {
            title,
            link,
            summary,
            date_pub,
        });
    }

    Ok(articles)
}
