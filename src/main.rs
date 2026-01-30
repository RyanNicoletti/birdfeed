mod article;
mod rss;
mod scrape;
mod source;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use source::Source;

#[get("/")]
async fn get_articles() -> impl Responder {
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
        }
    }
    println!("articles: {:#?}", all_articles);
    HttpResponse::Ok().body(format!("{:?}", all_articles))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(get_articles))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
