mod article;
mod rss;
mod scrape;
mod source;
use source::Source;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        let articles: Vec<article::Article> = source.fetch_articles().await?;
        all_articles.extend(articles);
    }
    println!("articles: {:#?}", all_articles);
    Ok(())
}
