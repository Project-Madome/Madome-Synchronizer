use super::Metadata;

#[derive(Debug, PartialEq)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub groups: Vec<String>,
    pub artists: Vec<String>,
    pub series: Vec<String>,
    pub tags: Vec<String>,
    pub characters: Vec<String>,
    pub language: String,
    pub content_type: String,
    pub created_at: String,
}
