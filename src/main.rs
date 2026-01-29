mod article;
mod rss;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let articles = rss::fetch_from_rss("https://www.cidrap.umn.edu/news/49/rss").await?;
    println!("articles: {articles:?}");
    Ok(())
}

// https://www.cdc.gov/bird-flu/spotlights/index.html
// https://www.who.int/emergencies/disease-outbreak-news
// https://www.cidrap.umn.edu/avian-influenza-bird-flu
// https://news.google.com/rss/search?q=avian+influenza&hl=en-US&gl=US&ceid=US:en
