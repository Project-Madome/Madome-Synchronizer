#[derive(Debug, PartialEq)]
pub enum Metadata {
    ID(Option<i32>),
    Title(Option<String>),
    Artists(Option<Vec<String>>),
    Series(Option<Vec<String>>),
    ContentType(Option<ContentType>),
    Language(Option<String>),
    Tags(Option<Vec<String>>),
    Groups(Option<Vec<String>>),
    Characters(Option<Vec<String>>),
    CreatedAt(Option<String>),
    ContentURL(Option<String>),
    ThumbnailURL(Option<String>),
}

impl Metadata {
    pub fn as_str(&self) -> &str {
        match self {
            Metadata::ID(_) => "ID",
            Metadata::Series(_) => "Series",
            Metadata::Language(_) => "Language",
            Metadata::Tags(_) => "Tags",
            Metadata::ContentType(_) => "Type",
            Metadata::Title(_) => "Title",
            Metadata::Artists(_) => "Artists",
            Metadata::Groups(_) => "Group",
            Metadata::Characters(_) => "Characters",
            Metadata::CreatedAt(_) => "CreatedAt",
            Metadata::ContentURL(_) => "ContentURL",
            Metadata::ThumbnailURL(_) => "ThumbnailURL",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ContentType {
    Manga,
    Doujinshi,
    ArtistCG,
}

impl ContentType {
    pub fn from(s: String) -> ContentType {
        match s.as_str() {
            "manga" => ContentType::Manga,
            "doujinshi" => ContentType::Doujinshi,
            "artist CG" => ContentType::ArtistCG,
            unknown => panic!("Unknown ContentType {}", unknown),
        }
    }
}
