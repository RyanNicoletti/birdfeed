use crate::article::Article;
use crate::rss;
use crate::scrape;

pub enum Source {
    RSS { url: String },
    PoultryWorld { url: String },
    WattAgNet { url: String },
}

impl Source {
    pub async fn fetch_articles(&self) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
        match self {
            Source::RSS { url } => rss::fetch_from_rss(url).await,
            Source::PoultryWorld { url } => scrape::poultryworld::fetch_from_pw(url).await,
            Source::WattAgNet { url } => scrape::wattagnet::fetch_from_wag(url).await,
        }
    }
}
