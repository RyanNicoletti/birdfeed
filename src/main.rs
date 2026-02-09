mod article;
mod db;
mod scrape;
mod source;
use actix_web::{App, HttpResponse, HttpServer, get, web};
use askama::Template;
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};

struct AppState {
    db_pool: sqlx::SqlitePool,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    dates: Vec<db::DateWithArticles>,
}

#[get("/")]
async fn index(data: web::Data<AppState>) -> HttpResponse {
    let dates = db::get_articles_by_pub_date(&data.db_pool)
        .await
        .unwrap_or_default();

    let template = IndexTemplate { dates };

    match template.render() {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
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
            Job::new_async("0 0 0,12 * * *", move |_uuid, _l| {
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

    HttpServer::new(move || App::new().app_data(pool.clone()).service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
