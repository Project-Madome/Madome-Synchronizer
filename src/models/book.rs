use super::Metadata;

#[derive(Debug, PartialEq)]
pub struct Book {
    pub id: Metadata,
    pub title: Metadata,
    pub groups: Metadata,
    pub artists: Metadata,
    pub series: Metadata,
    pub tags: Metadata,
    pub language: Metadata,
    pub content_type: Metadata,
    pub characters: Metadata,
    pub created_at: Metadata,
}
