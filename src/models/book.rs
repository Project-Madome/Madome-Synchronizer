use std::convert::Into;

use super::{ContentType, Language, Metadata};

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

#[derive(Debug, PartialEq)]
pub struct MetadataBook {
    pub id: Metadata,
    pub title: Metadata,
    pub groups: Metadata,
    pub artists: Metadata,
    pub series: Metadata,
    pub tags: Metadata,
    pub characters: Metadata,
    pub language: Metadata,
    pub content_type: Metadata,
    pub created_at: Metadata,
    pub thumbnail_url: Metadata,
}

impl From<MetadataBook> for Book {
    fn from(metadata_book: MetadataBook) -> Book {
        let content_type: ContentType = metadata_book.content_type.into();
        let language: Language = metadata_book.language.into();

        Book {
            id: metadata_book.id.into(),
            title: metadata_book.title.into(),
            groups: metadata_book.groups.into(),
            artists: metadata_book.artists.into(),
            series: metadata_book.series.into(),
            tags: metadata_book.tags.into(),
            characters: metadata_book.characters.into(),
            language: language.into(),
            content_type: content_type.into(),
            created_at: metadata_book.created_at.into(),
        }
    }
}
