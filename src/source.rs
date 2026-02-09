use crate::article::Article;
use crate::scrape;

pub enum Source {
    Cidrap { url: String },
    PoultryWorld { url: String },
    WattAgNet { url: String },
}

impl Source {
    pub async fn fetch_articles(&self) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
        match self {
            Source::Cidrap { url } => scrape::cidrap::fetch(url).await,
            Source::PoultryWorld { url } => scrape::poultryworld::fetch(url).await,
            Source::WattAgNet { url } => scrape::wattagnet::fetch(url).await,
        }
    }
}
