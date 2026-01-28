use chrono::{DateTime, Duration, Utc};
use rss::Channel;
use std::error::Error;

#[derive(Debug)]
struct Article {
    title: String,
    link: String,
    summary: String,
    date_pub: String,
}

#[tokio::main]
async fn main() {
    let channel = fetch_rss_feeds().await.unwrap();
    let mut articles: Vec<Article> = Vec::new();
    for chan in channel.items() {
        articles.push(Article {
            title: chan.title().unwrap_or("No title found").to_string(),
            link: chan.link().unwrap_or("No link found").to_string(),
            summary: chan.description().unwrap_or("No summary found").to_string(),
            date_pub: chan.pub_date().unwrap_or("No date found").to_string(),
        })
    }
    let new_articles = filter_articles(articles, 7);
    println!("articles: {new_articles:?}");
}

async fn fetch_rss_feeds() -> Result<Channel, Box<dyn Error>> {
    let body = reqwest::get("https://www.cidrap.umn.edu/news/49/rss")
        .await?
        .bytes()
        .await?;
    let channel = Channel::read_from(&body[..])?;
    Ok(channel)
}

fn filter_articles(articles: Vec<Article>, days: i64) -> Vec<Article> {
    let cut_off_date = Utc::now() - Duration::days(days);
    articles
        .into_iter()
        .filter(|article| {
            let pub_date = DateTime::parse_from_rfc2822(&article.date_pub);
            match pub_date {
                Ok(d) => d > cut_off_date,
                Err(_) => false,
            }
        })
        .collect()
}
// https://www.cdc.gov/bird-flu/spotlights/index.html
// https://www.who.int/emergencies/disease-outbreak-news
// https://www.cidrap.umn.edu/avian-influenza-bird-flu
// https://news.google.com/rss/search?q=avian+influenza&hl=en-US&gl=US&ceid=US:en
