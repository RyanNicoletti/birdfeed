use crate::article;
use chrono::{Duration, Local};
use sqlx::SqlitePool;
use sqlx::migrate::MigrateDatabase;

pub async fn create_db(db_url: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
    if !sqlx::Sqlite::database_exists(db_url).await? {
        sqlx::Sqlite::create_database(db_url).await?;
    }
    let pool = sqlx::SqlitePool::connect(db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn insert_posts(
    articles: Vec<article::Article>,
    db_pool: &SqlitePool,
) -> Result<u64, sqlx::Error> {
    let mut conn = db_pool.acquire().await?;
    let ts = chrono::offset::Local::now().to_rfc3339();
    let mut insert_count: u64 = 0;
    for a in articles {
        let inserted = sqlx::query!(
            r#"
            INSERT INTO articles (title, link, summary, date_pub, source, fetched_at)
            VALUES
            (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(title) DO NOTHING
            "#,
            a.title,
            a.link,
            a.summary,
            a.date_pub,
            a.source,
            ts
        )
        .execute(&mut *conn)
        .await?
        .rows_affected();
        insert_count = inserted + insert_count;
    }
    Ok(insert_count)
}

/// date + articles published on that date
#[derive(Debug)]
pub struct DateWithArticles {
    pub date: String,
    pub articles: Vec<article::Article>,
}

pub async fn get_articles_by_pub_date(
    db_pool: &SqlitePool,
) -> Result<Vec<DateWithArticles>, sqlx::Error> {
    let cutoff = (Local::now() - Duration::days(14))
        .format("%Y-%m-%d")
        .to_string();

    let articles = sqlx::query_as!(
        article::Article,
        r#"
        SELECT title, link, summary, date_pub, source, fetched_at
        FROM articles
        WHERE date_pub >= ?
        ORDER BY date_pub DESC
        "#,
        cutoff
    )
    .fetch_all(db_pool)
    .await?;

    let mut dates_map: std::collections::HashMap<String, Vec<article::Article>> =
        std::collections::HashMap::new();

    for article in articles {
        let date = article.date_pub.chars().take(10).collect::<String>();
        dates_map.entry(date).or_default().push(article);
    }

    let mut dates: Vec<DateWithArticles> = dates_map
        .into_iter()
        .map(|(date, articles)| DateWithArticles { date, articles })
        .collect();

    dates.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(dates)
}
