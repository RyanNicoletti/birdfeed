use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Article {
    pub title: String,
    pub link: String,
    pub summary: String,
    pub date_pub: String,
}
