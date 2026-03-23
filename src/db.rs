use crate::article::{self, Article};
use crate::error::AppError;
use chrono::{Duration, Local};
use sqlx::SqlitePool;
use sqlx::migrate::MigrateDatabase;

pub async fn create_db(db_url: &str) -> Result<sqlx::SqlitePool, AppError> {
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
) -> Result<u64, AppError> {
    let mut tx = db_pool.begin().await?;
    let ts = chrono::offset::Local::now().to_rfc3339();
    let mut insert_count: u64 = 0;

    for a in articles {
        let inserted = sqlx::query!(
            r#"
            INSERT INTO articles (title, link, summary, body, date_pub, source, fetched_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(title) DO NOTHING
            "#,
            a.title,
            a.link,
            a.summary,
            a.body,
            a.date_pub,
            a.source,
            ts
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        insert_count += inserted;
    }

    tx.commit().await?;
    Ok(insert_count)
}

pub async fn insert_summary(summary: String, db_pool: &SqlitePool) -> Result<(), AppError> {
    let mut conn = db_pool.acquire().await?;
    let now = Local::now();
    let end = now.format("%Y-%m-%d").to_string();
    let start = (now - Duration::days(7)).format("%Y-%m-%d").to_string();
    let date_range = format!("{} to {}", start, end);

    sqlx::query!(
        r#"
        INSERT INTO summaries (summary, date_range)
        VALUES (?1, ?2)
        ON CONFLICT(date_range) DO NOTHING
        "#,
        summary,
        date_range
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// date + articles published on that date
#[derive(Debug)]
pub struct DateWithArticles {
    pub date: String,
    pub articles: Vec<article::Article>,
}

pub async fn get_articles_by_pub_date(
    db_pool: &SqlitePool,
) -> Result<Vec<DateWithArticles>, AppError> {
    let cutoff = (Local::now() - Duration::days(14))
        .format("%Y-%m-%d")
        .to_string();

    let articles = sqlx::query_as!(
        article::Article,
        r#"
        SELECT title, link, summary, body, date_pub, source, fetched_at
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

pub async fn get_articles_for_summary(db_pool: &SqlitePool) -> Result<Vec<Article>, AppError> {
    let cutoff = (Local::now() - Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();

    let articles = sqlx::query_as!(
        article::Article,
        r#"
        SELECT title, link, summary, body, date_pub, source, fetched_at
        FROM articles
        WHERE date_pub >= ?
        ORDER BY date_pub DESC
        "#,
        cutoff
    )
    .fetch_all(db_pool)
    .await?;
    Ok(articles)
}

pub async fn get_latest_summary(db_pool: &SqlitePool) -> Result<Option<String>, AppError> {
    let summary = sqlx::query_scalar!(
        r#"
        SELECT summary
        FROM summaries
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(db_pool)
    .await?;
    Ok(summary)
}
