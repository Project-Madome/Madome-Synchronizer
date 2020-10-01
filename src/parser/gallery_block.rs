use time;

use anyhow;
use async_trait::async_trait;
use reqwest;
use scraper::{Html, Selector};

use crate::models::{Book, ContentType, Metadata};
use crate::parser::Parser;

/// Can't parse Groups
pub struct GalleryBlock {
    id: i32,
}

impl GalleryBlock {
    pub fn new(id: i32) -> GalleryBlock {
        GalleryBlock { id }
    }

    pub fn parse_single_metadata(&self, fragment: scraper::ElementRef) -> String {
        let anchor_selector = Selector::parse("a").unwrap();

        String::from(
            fragment
                .select(&anchor_selector)
                .next()
                .unwrap()
                .text()
                .next()
                .unwrap(),
        )
    }

    pub fn parse_multiple_metadata(&self, fragment: scraper::ElementRef) -> Vec<String> {
        let ul_selector = Selector::parse("ul").unwrap();
        let li_selector = Selector::parse("li").unwrap();

        fragment
            .select(&ul_selector)
            .next()
            .unwrap()
            .select(&li_selector)
            .map(|element| String::from(element.text().next().unwrap()))
            .collect::<Vec<_>>()
    }

    pub fn parse_title(&self, fragment: &Html) -> String {
        let title_selector = Selector::parse("h1.lillie > a").unwrap();

        String::from(
            fragment
                .select(&title_selector)
                .next()
                .unwrap()
                .text()
                .next()
                .unwrap(),
        )
    }

    pub fn is_nothing(&self, element: &scraper::ElementRef<'_>) -> bool {
        println!(
            "is nothing {}",
            String::from(element.text().next().unwrap()).trim()
        );
        String::from(element.text().next().unwrap()).trim() == String::from("N/A")
    }

    /// Change return type to Option<Vec<String>>
    /// and check N/A // 1722734
    pub fn parse_artists(&self, fragment: &Html) -> Option<Vec<String>> {
        let artist_list_selector = Selector::parse(".artist-list").unwrap();
        let ul_selector = Selector::parse("ul").unwrap();
        let li_selector = Selector::parse("li").unwrap();

        let artist_list = fragment.select(&artist_list_selector).next().unwrap();

        if self.is_nothing(&artist_list) {
            return None;
        }

        let ul = artist_list.select(&ul_selector).next().unwrap();

        Some(
            ul.select(&li_selector)
                .map(|element| String::from(element.text().next().unwrap()))
                .collect::<Vec<_>>(),
        )
    }

    pub fn parse_series(&self, element: scraper::ElementRef) -> Option<Vec<String>> {
        if self.is_nothing(&element) {
            return None;
        }

        Some(self.parse_multiple_metadata(element))
    }

    pub fn parse_tags(&self, element: scraper::ElementRef) -> Option<Vec<String>> {
        // <tr>
        // <td>Tags</td>
        // <td class="relatedtags">
        // <ul>
        // </ul>
        // </td>
        // </tr>
        /* if self.is_nothing(&element) {
            return None;
        } */

        let tags = self.parse_multiple_metadata(element);

        if tags.is_empty() {
            return None;
        }

        Some(tags)
    }

    pub fn parse_content_type(&self, element: scraper::ElementRef) -> Option<ContentType> {
        if self.is_nothing(&element) {
            return None;
        }

        Some(ContentType::from(self.parse_single_metadata(element)))
    }

    pub fn parse_language(&self, element: scraper::ElementRef) -> Option<String> {
        if self.is_nothing(&element) {
            return None;
        }

        Some(self.parse_single_metadata(element))
    }

    pub fn parse_created_at(&self, fragment: &Html) -> Option<String> {
        let date_selector = Selector::parse(".date").unwrap();

        let date_str = fragment
            .select(&date_selector)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .trim();

        let c = &date_str[..date_str.len() - 3];

        let cc = &date_str[c.len()..];

        let mut date_string = String::new();

        date_string.push_str(c);
        date_string.push(' ');
        date_string.push_str(cc);
        date_string.push_str(":00");

        Some(date_string)
    }

    pub fn parse_metadata(&self, fragment: &Html, metadata_type: Metadata) -> Metadata {
        match metadata_type {
            Metadata::Title(_) => Metadata::Title(Some(self.parse_title(fragment))),
            Metadata::Artists(_) => Metadata::Artists(self.parse_artists(fragment)),
            Metadata::CreatedAt(_) => Metadata::CreatedAt(self.parse_created_at(fragment)),
            _ => {
                let metadata_table_selector = Selector::parse(".dj-content > .dj-desc").unwrap();
                let tr_element_selector = Selector::parse("tr").unwrap();
                let td_element_selector = Selector::parse("td").unwrap();

                let r = fragment
                    .select(&metadata_table_selector)
                    .next()
                    .unwrap()
                    .select(&tr_element_selector)
                    .find(|element| {
                        let mut element = element.select(&td_element_selector);

                        element.next().unwrap().text().next().unwrap() == metadata_type.as_str()
                    })
                    .unwrap()
                    /*
                    <tr>
                        <td>Series</td>
                        <td>
                            N/A
                        </td>
                    </tr>
                    */
                    .select(&td_element_selector)
                    /*
                    0: <td>Series</td>
                    1: <td>N/A</td>
                    */
                    .nth(1)
                    .unwrap();

                let is_nothing =
                    String::from(r.text().next().unwrap()).trim() == String::from("N/A");

                if is_nothing {
                    // None
                    metadata_type
                } else {
                    match metadata_type {
                        Metadata::ContentType(_) => {
                            Metadata::ContentType(self.parse_content_type(r))
                        }
                        Metadata::Language(_) => Metadata::Language(self.parse_language(r)),
                        Metadata::Series(_) => Metadata::Series(self.parse_series(r)),
                        Metadata::Tags(_) => Metadata::Tags(self.parse_tags(r)),
                        _ => metadata_type,
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Parser for GalleryBlock {
    type RequestData = String;
    type ParseData = Book;

    fn url(&self) -> String {
        format!("https://ltn.hitomi.la/galleryblock/{}.html", self.id)
    }

    async fn request(&self) -> anyhow::Result<Self::RequestData> {
        let client = reqwest::Client::builder().build()?;

        let gallery_block_html = client.get(self.url().as_str()).send().await?.text().await?;

        Ok(gallery_block_html)
    }

    async fn parse(&self, request_data: Self::RequestData) -> anyhow::Result<Self::ParseData> {
        let fragment = Html::parse_fragment(request_data.as_str());

        let title = self.parse_metadata(&fragment, Metadata::Title(None));
        let artists = self.parse_metadata(&fragment, Metadata::Artists(None));
        let series = self.parse_metadata(&fragment, Metadata::Series(None));
        let tags = self.parse_metadata(&fragment, Metadata::Tags(None));
        let language = self.parse_metadata(&fragment, Metadata::Language(None));
        let content_type = self.parse_metadata(&fragment, Metadata::ContentType(None));
        let created_at = self.parse_metadata(&fragment, Metadata::CreatedAt(None));

        let book = Book {
            id: Metadata::Id(Some(self.id)),
            title,
            artists,
            groups: Metadata::Groups(None),
            characters: Metadata::Characters(None),
            series,
            tags,
            language,
            content_type,
            created_at,
        };

        Ok(book)
    }
}

#[cfg(test)]
mod tests {
    use scraper::Html;

    use super::Book;
    use super::ContentType;
    use super::GalleryBlock;
    use super::Metadata;
    use super::Parser;

    #[tokio::test]
    async fn parse() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1724122);

        let rd = gallery_block.request().await?;

        let pd = gallery_block.parse(rd).await?;

        let expected = Book {
            id: Metadata::Id(Some(1724122)),
            title: Metadata::Title(Some(String::from("Tsundere Imouto | 츤데레 여동생"))),
            artists: Metadata::Artists(Some(vec![String::from("airandou")])),
            series: Metadata::Series(None),
            groups: Metadata::Groups(None),
            characters: Metadata::Characters(None),
            tags: Metadata::Tags(Some(
                ["footjob ♀", "loli ♀", "sister ♀", "incest"]
                    .iter()
                    .map(|st| String::from(*st))
                    .collect::<Vec<_>>(),
            )),
            language: Metadata::Language(Some(String::from("한국어"))),
            content_type: Metadata::ContentType(Some(ContentType::Manga)),
            created_at: Metadata::CreatedAt(Some(String::from("2020-09-02 10:01:00 -05:00"))),
        };

        assert_eq!(expected.id, pd.id);
        assert_eq!(expected.title, pd.title);
        assert_eq!(expected.artists, pd.artists);
        assert_eq!(expected.series, pd.series);
        assert_eq!(expected.groups, pd.groups);
        assert_eq!(expected.characters, pd.characters);
        assert_eq!(expected.tags, pd.tags);
        assert_eq!(expected.language, pd.language);
        assert_eq!(expected.content_type, pd.content_type);
        assert_eq!(expected.created_at, pd.created_at);

        Ok(())
    }

    #[tokio::test]
    async fn parse_title() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1399900);

        let rd = gallery_block.request().await?;

        let fragment = Html::parse_fragment(rd.as_str());

        let title = gallery_block.parse_title(&fragment);

        let expected = String::from("COMIC LO 2019-05");

        assert_eq!(expected, title);

        Ok(())
    }

    #[tokio::test]
    async fn parse_artists() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1399900);

        let rd = gallery_block.request().await?;

        let fragment = Html::parse_fragment(rd.as_str());

        let artists = gallery_block.parse_metadata(&fragment, Metadata::Artists(None));

        let expected = Metadata::Artists(Some(
            [
                "airandou",
                "atage",
                "hayake",
                "isawa nohri",
                "kinomoto anzu",
                "maeshima ryou",
                "mdo-h",
                "nadadekoko",
                "nekodanshaku",
                "noise",
                "ryoumoto hatsumi",
                "sabaku",
                "shiratama moti",
                "takamichi",
                "ueda yuu",
                "usakun",
                "yamaya oowemon",
                "yusa",
            ]
            .iter()
            .map(|st| String::from(*st))
            .collect::<Vec<_>>(),
        ));

        assert_eq!(expected, artists);

        Ok(())
    }

    #[tokio::test]
    async fn parse_artists_is_nothing() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1722267);

        let rd = gallery_block.request().await?;

        let fragment = Html::parse_fragment(rd.as_str());

        let artists = gallery_block.parse_metadata(&fragment, Metadata::Artists(None));

        let expected = Metadata::Artists(None);

        assert_eq!(expected, artists);

        Ok(())
    }

    #[tokio::test]
    async fn parse_language() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1399900);

        let rd = gallery_block.request().await?;

        let fragment = Html::parse_fragment(rd.as_str());

        let content_type = gallery_block.parse_metadata(&fragment, Metadata::Language(None));

        let expected = Metadata::Language(Some(String::from("한국어")));

        assert_eq!(expected, content_type);

        Ok(())
    }

    #[tokio::test]
    async fn parse_content_type() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1399900);

        let rd = gallery_block.request().await?;

        let fragment = Html::parse_fragment(rd.as_str());

        let content_type = gallery_block.parse_metadata(&fragment, Metadata::ContentType(None));

        let expected = Metadata::ContentType(Some(ContentType::Manga));

        assert_eq!(expected, content_type);

        Ok(())
    }

    #[tokio::test]
    async fn parse_series() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1277807);

        let rd = gallery_block.request().await?;

        let fragment = Html::parse_fragment(rd.as_str());

        let series = gallery_block.parse_metadata(&fragment, Metadata::Series(None));

        let e = [
            "eromanga sensei",
            "nier automata",
            "ranma 12",
            "sword art online",
            "the idolmaster",
            "the melancholy of haruhi suzumiya",
            "to love-ru",
            "urusei yatsura",
            "yuru camp",
        ]
        .iter()
        .map(|s| String::from(*s))
        .collect::<Vec<_>>();

        let expected = Metadata::Series(Some(e));

        assert_eq!(expected, series);

        Ok(())
    }

    #[tokio::test]
    async fn parse_series_is_nothing() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1399900);

        let rd = gallery_block.request().await?;

        let fragment = Html::parse_fragment(rd.as_str());

        let series_nothing = gallery_block.parse_metadata(&fragment, Metadata::Series(None));

        let expected = Metadata::Series(None);

        assert_eq!(expected, series_nothing);

        Ok(())
    }

    #[tokio::test]
    async fn parse_tags() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1724122);

        let rd = gallery_block.request().await?;

        let fragment = Html::parse_fragment(rd.as_str());

        let tags = gallery_block.parse_metadata(&fragment, Metadata::Tags(None));

        let expected = Metadata::Tags(Some(
            ["footjob ♀", "loli ♀", "sister ♀", "incest"]
                .iter()
                .map(|st| String::from(*st))
                .collect::<Vec<_>>(),
        ));

        assert_eq!(expected, tags);

        Ok(())
    }

    #[tokio::test]
    async fn parse_tags_is_nothing() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1686905);

        let rd = gallery_block.request().await?;

        let fragment = Html::parse_fragment(rd.as_str());

        let tags = gallery_block.parse_metadata(&fragment, Metadata::Tags(None));

        let expected = Metadata::Tags(None);

        assert_eq!(expected, tags);

        Ok(())
    }

    #[tokio::test]
    async fn parse_created_at() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1724122);

        let rd = gallery_block.request().await?;

        let fragment = Html::parse_fragment(rd.as_str());

        let created_at = gallery_block.parse_metadata(&fragment, Metadata::CreatedAt(None));

        let expected = Metadata::CreatedAt(Some(String::from("2020-09-02 10:01:00 -05:00")));

        assert_eq!(expected, created_at);

        Ok(())
    }
}
