mod article;
mod rss;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rss_urls: [&str; 1] = ["https://www.cidrap.umn.edu/news/49/rss"];
    let mut all_articles: Vec<article::Article> = Vec::new();

    for url in rss_urls {
        let articles: Vec<article::Article> = rss::fetch_from_rss(url).await?;
        all_articles.extend(articles);
    }
    println!("articles: {all_articles:?}");
    Ok(())
}
