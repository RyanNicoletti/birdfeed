use crate::article;
use chrono;
use serde::Serialize;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Row, SqlitePool};

pub async fn create_db(db_url: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
    if !sqlx::Sqlite::database_exists(db_url).await? {
        sqlx::Sqlite::create_database(db_url).await?;
    }
    let pool = sqlx::SqlitePool::connect(db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn get_posts(db_pool: &SqlitePool) -> Result<Vec<article::Article>, sqlx::Error> {
    let articles = sqlx::query_as!(
        article::Article,
        r#"SELECT title, link, summary, date_pub, source, fetched_at FROM articles ORDER BY fetched_at"#
    )
    .fetch_all(db_pool)
    .await?;
    Ok(articles)
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

#[derive(Debug, Serialize)]
pub struct DateWithCount {
    pub date: String,
    pub count: i64,
}

pub async fn get_all_dates(db_pool: &SqlitePool) -> Result<Vec<DateWithCount>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            substr(fetched_at, 1, 10) as date,
            COUNT(*) as count
        FROM articles
        GROUP BY substr(fetched_at, 1, 10)
        ORDER BY substr(fetched_at, 1, 10) DESC
        "#,
    )
    .fetch_all(db_pool)
    .await?;
    let dates = rows
        .iter()
        .map(|row| DateWithCount {
            date: row.get("date"),
            count: row.get("count"),
        })
        .collect();
    Ok(dates)
}

pub async fn get_articles_by_date(
    db_pool: &SqlitePool,
    date: &str,
) -> Result<Vec<article::Article>, sqlx::Error> {
    let pattern = format!("{}%", date);
    let articles = sqlx::query_as!(
        article::Article,
        r#"
        SELECT title, link, summary, date_pub, source, fetched_at
        FROM articles
        WHERE fetched_at LIKE ?
        ORDER BY fetched_at
        "#,
        pattern
    )
    .fetch_all(db_pool)
    .await?;
    Ok(articles)
}
