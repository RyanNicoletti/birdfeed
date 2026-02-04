mod article;
mod db;
mod rss;
mod scrape;
mod source;
use actix_files as fs;
use actix_web::{App, Error, HttpRequest, HttpServer, Responder, Result, get, web};
use serde::Serialize;
use source::Source;
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};

struct AppState {
    db_pool: sqlx::SqlitePool,
}

#[derive(Serialize)]
struct PostArticlesResponse {
    status: String,
    count: u64,
}

#[get("/")]
async fn index(_req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let path: std::path::PathBuf = "./assets/index.html".parse().unwrap();
    Ok(fs::NamedFile::open(path)?)
}

async fn post_articles(
    data: web::Data<AppState>,
) -> Result<web::Json<PostArticlesResponse>, Error> {
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
    let mut all_articles: Vec<article::Article> = Vec::new();

    for source in sources {
        match source.fetch_articles().await {
            Ok(articles) => all_articles.extend(articles),
            Err(e) => eprintln!("Failed to fetch articles: {}", e),
        };
    }
    let num_inserted = db::insert_posts(all_articles, &data.db_pool)
        .await
        .expect("Error inserting articles into db");
    Ok(web::Json(PostArticlesResponse {
        status: "success".to_string(),
        count: num_inserted,
    }))
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

    let mut sched = JobScheduler::new().await.expect("couldnt feth articles???");

    sched
        .add(
            Job::new_async("0 0 0 * * *", move |_uuid, _l| {
                let db_pool = pool_for_cron.clone();
                Box::pin(async move {
                    match article::post_articles(&db_pool).await {
                        Ok(num) => println!("Articles fetched: {}", num),
                        Err(e) => eprintln!("Failed to fetch articles: {}", e),
                    }
                })
            })
            .expect("fuck"),
        )
        .await
        .expect("FUCK");

    sched.start().await.expect("bruh");

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
