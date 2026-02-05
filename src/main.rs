mod article;
mod db;
mod rss;
mod scrape;
mod source;
use actix_files as fs;
use actix_web::{App, Error, HttpRequest, HttpServer, Responder, Result, get, web};
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};

struct AppState {
    db_pool: sqlx::SqlitePool,
}

#[get("/")]
async fn index(_req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let path: std::path::PathBuf = "./assets/index.html".parse().unwrap();
    Ok(fs::NamedFile::open(path)?)
}

#[get("/api/get_articles")]
async fn get_articles(data: web::Data<AppState>) -> Result<impl Responder> {
    let articles = db::get_posts(&data.db_pool)
        .await
        .expect("Error getting articles from db.");

    Ok(web::Json(articles))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("Database url not set");
    let db_pool = db::create_db(&db_url)
        .await
        .expect("Error creating the database");
    let pool_for_cron = db_pool.clone();
    let pool = web::Data::new(AppState { db_pool });

    let scheduler = JobScheduler::new().await.expect("");

    scheduler
        .add(
            Job::new_async("7 7 7 * * *", move |_uuid, _l| {
                let db_pool = pool_for_cron.clone();
                Box::pin(async move {
                    match article::post_articles(&db_pool).await {
                        Ok(num) => println!("Articles fetched: {}", num),
                        Err(e) => eprintln!("Failed to fetch articles: {}", e),
                    }
                })
            })
            .expect("Unexpected error scheduling new async cron job."),
        )
        .await
        .expect("Unexpected error adding a new scheduled job");

    scheduler
        .start()
        .await
        .expect("Unexpected error starting the cron job");

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .service(index)
            .service(get_articles)
            .service(fs::Files::new("/assets", "./assets"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
