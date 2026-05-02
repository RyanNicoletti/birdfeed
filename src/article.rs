use crate::db;
use crate::error::AppError;
use crate::source::Source;
use serde::Serialize;
use sqlx::{Pool, Sqlite};

const KEYWORDS: &[&str] = &["flu", "influenza", "hpai", "h5"];

#[derive(Debug, Clone, Serialize)]
pub struct Article {
    pub title: String,
    pub link: String,
    pub summary: String,
    pub body: Option<String>,
    pub date_pub: String,
    pub source: String,
    pub fetched_at: String,
}

pub async fn post_articles(db_pool: &Pool<Sqlite>) -> Result<u64, AppError> {
    let sources: Vec<Source> = vec![
        Source::Cidrap {
            url: "https://www.cidrap.umn.edu/news/49/rss".to_string(),
        },
        Source::PoultryWorld {
            url: "https://www.poultryworld.net/".to_string(),
        },
        Source::WattAgNet {
            url: "https://news.google.com/rss/search?q=site:wattagnet.com+avian+influenza&hl=en-US&gl=US&ceid=US:en".to_string(),
        },
        Source::PoultrySite {
            url: "https://www.thepoultrysite.com/articles.rss".to_string(),
        },
    ];
    let mut all_articles: Vec<Article> = Vec::new();

    for source in sources {
        match source.fetch_articles().await {
            Ok(articles) => all_articles.extend(articles),
            Err(e) => eprintln!("Failed to fetch articles from {:?}: {}", source, e),
        };
    }
    let filtered_articles: Vec<Article> = all_articles
        .into_iter()
        .filter(|a| KEYWORDS.iter().any(|w| a.title.to_lowercase().contains(w)))
        .collect();

    let num_inserted = db::insert_posts(filtered_articles, db_pool).await?;
    Ok(num_inserted)
}
