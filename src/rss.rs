use crate::article::Article;
use chrono::{DateTime, Duration, Utc};
use rss::Channel;

pub async fn fetch_from_rss(url: &str) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    let body = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&body[..])?;
    let mut articles: Vec<Article> = Vec::new();
    for chan in channel.items() {
        articles.push(Article {
            title: chan.title().unwrap_or("No title found").to_string(),
            link: chan.link().unwrap_or("No link found").to_string(),
            summary: chan.description().unwrap_or("No summary found").to_string(),
            date_pub: chan.pub_date().unwrap_or("No date found").to_string(),
            source: url.to_owned(),
        })
    }

    let cut_off_date = Utc::now() - Duration::days(7);
    let filterted_articles = articles
        .into_iter()
        .filter(|article| {
            let pub_date = DateTime::parse_from_rfc2822(&article.date_pub);
            match pub_date {
                Ok(d) => d > cut_off_date,
                Err(_) => false,
            }
        })
        .collect();
    Ok(filterted_articles)
}
