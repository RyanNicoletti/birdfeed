use crate::article;
use chrono;
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
            INSERT INTO articles (title, link, summary, date_pub, fetched_at)
            VALUES
            (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(title) DO NOTHING
            "#,
            a.title,
            a.link,
            a.summary,
            a.date_pub,
            ts
        )
        .execute(&mut *conn)
        .await?
        .rows_affected();
        insert_count = inserted + insert_count;
    }
    Ok(insert_count)
}
