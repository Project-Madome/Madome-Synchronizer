#[derive(Debug, PartialEq, Eq)]
pub enum Metadata {
    ID(Option<i32>),
    Title(Option<String>),
    Artists(Option<Vec<String>>),
    Series(Option<Vec<String>>),
    ContentType(Option<ContentType>),
    Language(Option<Language>),
    Tags(Option<Vec<String>>),
    Groups(Option<Vec<String>>),
    Characters(Option<Vec<String>>),
    CreatedAt(Option<String>),
    ContentURL(Option<String>),
    ThumbnailURL(Option<String>),
}

impl<'a> Metadata {
    pub fn as_str(&self) -> &'a str {
        self.into()
    }
}

impl<'a> Into<&'a str> for &Metadata {
    fn into(self) -> &'a str {
        match self {
            Metadata::ID(_) => "ID",
            Metadata::Title(_) => "Title",
            Metadata::Artists(_) => "Artists",
            Metadata::Groups(_) => "Group",
            Metadata::Series(_) => "Series",
            Metadata::Characters(_) => "Characters",
            Metadata::Tags(_) => "Tags",
            Metadata::Language(_) => "Language",
            Metadata::ContentType(_) => "Type",
            Metadata::CreatedAt(_) => "CreatedAt",
            Metadata::ContentURL(_) => "ContentURL",
            Metadata::ThumbnailURL(_) => "ThumbnailURL",
        }
    }
}

impl Into<i32> for Metadata {
    fn into(self) -> i32 {
        match self {
            Metadata::ID(Some(id)) => id,
            _ => panic!("Failed Metadata into() -> i32, {:?}", self),
        }
    }
}

impl Into<String> for Metadata {
    fn into(self) -> String {
        match self {
            Self::Title(Some(title)) => title,
            Self::ThumbnailURL(Some(thumbnail_url)) => thumbnail_url,
            Self::CreatedAt(Some(created_at)) => created_at,

            Self::Language(None) => "N/A".to_string(),
            _ => panic!("Failed Metadata into() -> String, {:?}", self),
        }
    }
}

impl Into<Language> for Metadata {
    fn into(self) -> Language {
        match self {
            Self::Language(Some(langauge)) => langauge,
            _ => panic!("Failed Metadata into() -> Language, {:?}", self),
        }
    }
}

impl Into<ContentType> for Metadata {
    fn into(self) -> ContentType {
        match self {
            Self::ContentType(Some(content_type)) => content_type,
            _ => panic!("Failed Metadata into() -> ContentType, {:?}", self),
        }
    }
}

impl Into<Vec<String>> for Metadata {
    fn into(self) -> Vec<String> {
        match self {
            Self::Artists(Some(artists)) => artists,
            Self::Groups(Some(groups)) => groups,
            Self::Series(Some(series)) => series,
            Self::Characters(Some(characters)) => characters,
            Self::Tags(Some(tags)) => tags,

            Self::Artists(None) => vec![],
            Self::Groups(None) => vec![],
            Self::Series(None) => vec![],
            Self::Characters(None) => vec![],
            Self::Tags(None) => vec![],
            _ => panic!("Failed Metadata into() -> Vec<String>, {:?}", self),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Language {
    All,
    Korean,
    Japanese,
    English,
    Spanish,
    Thai,
    German,
    Chinese,
    Portuguese,
    French,
    Tagalog,
    Russian,
    Italian,
    Polish,
    Vietnamese,
    Hungarian,
    Czech,
    Indonesian,
    Arabic,
}

impl From<&str> for Language {
    fn from(lang: &str) -> Language {
        match lang {
            "한국어" => Language::Korean,
            "日本語" => Language::Japanese,
            "English" => Language::English,
            "Español" => Language::Spanish,
            "ไทย" => Language::Thai,
            "Deutsch" => Language::German,
            "中文" => Language::Chinese,
            "Português" => Language::Portuguese,
            "Français" => Language::French,
            "Tagalog" => Language::Tagalog,
            "Русский" => Language::Russian,
            "Italiano" => Language::Italian,
            "polski" => Language::Polish,
            "tiếng việt" => Language::Vietnamese,
            "magyar" => Language::Hungarian,
            "Čeština" => Language::Czech,
            "Bahasa Indonesia" => Language::Indonesian,
            "العربية" => Language::Arabic,

            _ => panic!("Can't Language from &str"),
        }
    }
}

impl<'a> Into<&'a str> for Language {
    fn into(self) -> &'a str {
        match self {
            Self::All => "all",
            Self::Korean => "korean",
            Self::Japanese => "japanese",
            Self::English => "english",
            Self::Spanish => "spanish",
            Self::Thai => "thai",
            Self::German => "german",
            Self::Chinese => "chinese",
            Self::Portuguese => "portuguese",
            Self::French => "french",
            Self::Tagalog => "tagalog",
            Self::Russian => "russian",
            Self::Italian => "italian",
            Self::Polish => "polish",
            Self::Vietnamese => "vietnamese",
            Self::Hungarian => "hungarian",
            Self::Czech => "czech",
            Self::Indonesian => "indonesian",
            Self::Arabic => "arabic",
            // _ => panic!("Can't Language into &str"),
        }
    }
}

impl Into<String> for Language {
    fn into(self) -> String {
        let st: &str = self.into();

        st.to_string()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ContentType {
    Manga,
    Doujinshi,
    ArtistCG,
}

impl Into<String> for ContentType {
    fn into(self) -> String {
        match self {
            Self::Manga => "manga".to_string(),
            Self::ArtistCG => "artist cg".to_string(),
            Self::Doujinshi => "doujinshi".to_string(),
        }
    }
}

impl ToString for ContentType {
    fn to_string(&self) -> String {
        match self {
            Self::Manga => "manga".to_string(),
            Self::ArtistCG => "artist cg".to_string(),
            Self::Doujinshi => "doujinshi".to_string(),
        }
    }
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
