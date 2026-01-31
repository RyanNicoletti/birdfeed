mod article;
mod rss;
mod scrape;
mod source;
use actix_files as fs;
use actix_web::{App, Error, HttpRequest, HttpServer, Responder, Result, get, web};
use source::Source;

#[get("/")]
async fn index(_req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let path: std::path::PathBuf = "./assets/index.html".parse().unwrap();
    Ok(fs::NamedFile::open(path)?)
}

#[get("/api/get_articles")]
async fn get_articles(_req: HttpRequest) -> Result<impl Responder> {
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

    Ok(web::Json(all_articles))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(get_articles)
            .service(fs::Files::new("/assets", "./assets"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
