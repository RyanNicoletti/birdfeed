use sqlx::migrate::MigrateDatabase;

pub async fn create_db(db_url: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
    if !sqlx::Sqlite::database_exists(db_url).await? {
        sqlx::Sqlite::create_database(db_url).await?;
    }
    let pool = sqlx::SqlitePool::connect(db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
