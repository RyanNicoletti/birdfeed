use crate::article::Article;
use crate::error::AppError;
use crate::scrape;

#[derive(Debug)]
pub enum Source {
    Cidrap { url: String },
    PoultryWorld { url: String },
    WattAgNet { url: String },
    PoultrySite { url: String },
}

impl Source {
    pub async fn fetch_articles(&self) -> Result<Vec<Article>, AppError> {
        match self {
            Source::Cidrap { url } => scrape::cidrap::fetch(url).await,
            Source::PoultryWorld { url } => scrape::poultryworld::fetch(url).await,
            Source::WattAgNet { url } => scrape::wattagnet::fetch(url).await,
            Source::PoultrySite { url } => scrape::poultrysite::fetch(url).await,
        }
    }
}
