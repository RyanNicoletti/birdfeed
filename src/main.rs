mod article;
mod db;
mod error;
mod llm;
mod scrape;
mod source;
use crate::db::{get_articles_for_summary, insert_summary};
use actix_web::{App, HttpResponse, HttpServer, get, web};
use askama::Template;
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::llm::summarize_articles;

struct AppState {
    db_pool: sqlx::SqlitePool,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    dates: Vec<db::DateWithArticles>,
    summary: String,
}

#[get("/")]
async fn index(data: web::Data<AppState>) -> HttpResponse {
    let dates = db::get_articles_by_pub_date(&data.db_pool)
        .await
        .unwrap_or_default();
    let summary = db::get_latest_summary(&data.db_pool)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to fetch latest summary: {}", e);
            None
        })
        .unwrap_or_default();
    let template = IndexTemplate { dates, summary };

    match template.render() {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprint!("DATABASE_URL not set in environment");
        std::process::exit(1);
    });
    let db_pool = db::create_db(&db_url).await.unwrap_or_else(|e| {
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    });
    let pool_for_cron = db_pool.clone();
    let pool_for_summarizer = db_pool.clone();
    let pool = web::Data::new(AppState { db_pool });

    let scheduler = JobScheduler::new()
        .await
        .expect("Failed to create job scheduler.");

    scheduler
        .add(
            Job::new_async("0 0 0,12 * * *", move |_uuid, _l| {
                let db_pool = pool_for_cron.clone();
                Box::pin(async move {
                    match article::post_articles(&db_pool).await {
                        Ok(num) => println!("Articles fetched: {}", num),
                        Err(e) => eprintln!("Failed to fetch articles: {}", e),
                    }
                })
            })
            .expect("Unexpected error scheduling post articles cron."),
        )
        .await
        .expect("Unexpected error adding post articles job to scheduler.");
    scheduler
        .add(
            // 0 0 13 * * 2
            Job::new_async("0 0 13 * * 2", move |_uuid, _l| {
                let db_pool = pool_for_summarizer.clone();
                Box::pin(async move {
                    let articles = match get_articles_for_summary(&db_pool).await {
                        Ok(a) => a,
                        Err(e) => {
                            eprintln!("Failed to fetch articles for summary: {}", e);
                            return;
                        }
                    };
                    let summary = match summarize_articles(&articles).await {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Failed to summarize articles: {}", e);
                            return;
                        }
                    };
                    if let Err(e) = insert_summary(summary, &db_pool).await {
                        eprintln!("Failed to insert article summary into db: {}", e);
                    }
                })
            })
            .expect("Unexpected error scheduling the summarizer cron."),
        )
        .await
        .expect("Unexpected error adding the summarizer cron to scheduler.");

    scheduler
        .start()
        .await
        .expect("Unexpected error starting the cron job");

    HttpServer::new(move || App::new().app_data(pool.clone()).service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
