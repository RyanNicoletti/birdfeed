use crate::db;
use crate::source::Source;
use serde::Serialize;
use sqlx::{Pool, Sqlite};

const KEYWORDS: &[&str] = &["flu", "influenza", "hpai"];

#[derive(Debug, Clone, Serialize)]
pub struct Article {
    pub title: String,
    pub link: String,
    pub summary: String,
    pub date_pub: String,
    pub source: String,
    pub fetched_at: String,
}

pub async fn post_articles(db_pool: &Pool<Sqlite>) -> Result<u64, Box<dyn std::error::Error>> {
    let sources: Vec<Source> = vec![
        Source::RSS {
            url: "https://www.cidrap.umn.edu/news/49/rss".to_string(),
        },
        Source::PoultryWorld {
            url: "https://www.poultryworld.net/".to_string(),
        },
        Source::WattAgNet {
            url: "https://www.wattagnet.com/broilers-turkeys/diseases-health".to_string(),
        },
    ];
    let mut all_articles: Vec<Article> = Vec::new();

    for source in sources {
        match source.fetch_articles().await {
            Ok(articles) => all_articles.extend(articles),
            Err(e) => eprintln!("Failed to fetch articles: {}", e),
        };
    }
    let filtered_articles: Vec<Article> = all_articles
        .into_iter()
        .filter(|a| KEYWORDS.iter().any(|w| a.title.to_lowercase().contains(w)))
        .collect();

    let num_inserted = db::insert_posts(filtered_articles, db_pool)
        .await
        .expect("Error inserting articles into db");
    Ok(num_inserted)
}
