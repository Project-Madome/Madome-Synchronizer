use anyhow;
use log::trace;
use madome_client::book::{ContentType, Language, Metadata, MetadataBook};
use reqwest;
use scraper::{Html, Selector};

use crate::parser::Parser;

/// Can't parse Groups, Characters
pub struct GalleryBlock {
    id: u32,
    request_data: Option<Box<String>>,
}

impl GalleryBlock {
    pub fn new(id: u32) -> GalleryBlock {
        GalleryBlock {
            id,
            request_data: None,
        }
    }

    pub fn parse_single_metadata(&self, element: scraper::ElementRef) -> String {
        let anchor_selector = Selector::parse("a").unwrap();

        element
            .select(&anchor_selector)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .to_string()
    }

    pub fn parse_multiple_metadata(&self, element: scraper::ElementRef) -> Vec<String> {
        let ul_selector = Selector::parse("ul").unwrap();
        let li_selector = Selector::parse("li").unwrap();

        element
            .select(&ul_selector)
            .next()
            .unwrap()
            .select(&li_selector)
            .map(|element| element.text().next().unwrap().to_string())
            .collect::<Vec<_>>()
    }

    pub fn parse_title(&self, fragment: &Html) -> String {
        let title_selector = Selector::parse("h1.lillie > a").unwrap();

        fragment
            .select(&title_selector)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .to_string()
    }

    pub fn is_nothing(&self, element: &scraper::ElementRef<'_>) -> bool {
        element.text().next().unwrap().trim() == "N/A"
    }

    /// Change return type to Option<Vec<String>>
    /// and check N/A
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
                .map(|element| element.text().next().unwrap().to_string())
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

    pub fn parse_language(&self, element: scraper::ElementRef) -> Option<Language> {
        if self.is_nothing(&element) {
            return None;
        }

        Some(Language::from(self.parse_single_metadata(element).as_str()))
    }

    /// uncomment on v2 api
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

        /* let c = &date_str[..date_str.len() - 3];

        let cc = &date_str[c.len()..];

        let mut date_string = String::new();

        date_string.push_str(c);
        date_string.push(' ');
        date_string.push_str(cc);
        date_string.push_str(":00"); */

        Some(date_str.to_string())
    }

    pub fn parse_thumbnail_url(&self, fragment: &Html) -> String {
        let anchor_selector = Selector::parse("a").unwrap();
        let img_selector = Selector::parse("img").unwrap();

        let anchor = fragment.select(&anchor_selector).next().unwrap();

        anchor
            .select(&img_selector)
            .next()
            .unwrap()
            .value()
            .attr("src")
            .unwrap()[("//tn.hitomi.la").len()..]
            .replace("smallbig", "big")
    }

    #[deprecated]
    pub fn parse_content_url(&self, fragment: &Html) -> String {
        let anchor_selector = Selector::parse("a").unwrap();

        fragment
            .select(&anchor_selector)
            .next()
            .unwrap()
            .value()
            .attr("href")
            .unwrap()
            .to_string()
    }

    pub fn parse_metadata(&self, fragment: &Html, metadata_type: Metadata) -> Metadata {
        match metadata_type {
            Metadata::Title(_) => Metadata::Title(Some(self.parse_title(fragment))),
            Metadata::Artists(_) => Metadata::Artists(self.parse_artists(fragment)),
            Metadata::CreatedAt(_) => Metadata::CreatedAt(self.parse_created_at(fragment)),
            //  Metadata::ContentURL(_) => Metadata::ContentURL(Some(self.parse_content_url(fragment))),
            Metadata::ThumbnailURL(_) => {
                Metadata::ThumbnailURL(Some(self.parse_thumbnail_url(fragment)))
            }
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
                        let element = element.select(&td_element_selector).next().unwrap();

                        element.text().next().unwrap() == metadata_type.as_str()
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

                match metadata_type {
                    Metadata::ContentType(_) => Metadata::ContentType(self.parse_content_type(r)),
                    Metadata::Language(_) => Metadata::Language(self.parse_language(r)),
                    Metadata::Series(_) => Metadata::Series(self.parse_series(r)),
                    Metadata::Tags(_) => Metadata::Tags(self.parse_tags(r)),
                    _ => metadata_type,
                }
            }
        }
    }
}

impl Parser for GalleryBlock {
    type RequestData = String;
    type ParseData = MetadataBook;

    fn request_data(&self) -> anyhow::Result<&Box<Self::RequestData>> {
        trace!("GalleryBlock::request_data()");
        match self.request_data {
            Some(ref rd) => Ok(rd),
            None => Err(anyhow::Error::msg("Can't get request_data")),
        }
    }

    fn url(&self) -> anyhow::Result<String> {
        trace!("GalleryBlock::url()");
        Ok(format!(
            "https://ltn.hitomi.la/galleryblock/{}.html",
            self.id
        ))
    }

    fn request(mut self) -> anyhow::Result<Box<Self>> {
        trace!("GalleryBlock::request()");
        let client = reqwest::blocking::Client::builder().build()?;

        let gallery_block_html = client.get(&self.url()?).send()?.text()?;

        self.request_data = Some(Box::new(gallery_block_html));

        Ok(Box::new(self))
    }

    fn parse(&self) -> anyhow::Result<Self::ParseData> {
        trace!("GalleryBlock::parse()");
        let fragment = Html::parse_fragment(&self.request_data()?);

        let id = Metadata::ID(Some(self.id));
        let title = self.parse_metadata(&fragment, Metadata::Title(None));
        let artists = self.parse_metadata(&fragment, Metadata::Artists(None));
        let series = self.parse_metadata(&fragment, Metadata::Series(None));
        let tags = self.parse_metadata(&fragment, Metadata::Tags(None));
        let language = self.parse_metadata(&fragment, Metadata::Language(None));
        let content_type = self.parse_metadata(&fragment, Metadata::ContentType(None));
        let created_at = self.parse_metadata(&fragment, Metadata::CreatedAt(None));
        let thumbnail_url = self.parse_metadata(&fragment, Metadata::ThumbnailURL(None));
        // let content_url = self.parse_metadata(&fragment, Metadata::ContentURL(None));

        let metadata_book = MetadataBook {
            id,
            title,
            artists,
            series,
            tags,
            language,
            content_type,
            created_at,
            thumbnail_url,
            characters: Metadata::Characters(None),
            groups: Metadata::Groups(None),
            page_count: Metadata::Page(None),
        };

        Ok(metadata_book)
    }
}

#[cfg(test)]
mod tests {
    use scraper::Html;

    use super::ContentType;
    use super::GalleryBlock;
    use super::Language;
    use super::Metadata;
    use super::Parser;

    /* #[test]
     fn parse_gallery_block() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1724122);

        let rd = gallery_block.request()?;

        let pd = gallery_block.parse(rd)?;

        let expected = BookOfGalleryBlock {
            id: Metadata::ID(Some(1724122)),
            title: Metadata::Title(Some("Tsundere Imouto | 츤데레 여동생".to_string())),
            artists: Metadata::Artists(Some(vec!["airandou".to_string()])),
            series: Metadata::Series(None),
            // groups: Metadata::Groups(None),
            // characters: Metadata::Characters(None),
            tags: Metadata::Tags(Some(
                ["footjob ♀", "loli ♀", "sister ♀", "incest"]
                    .iter()
                    .map(|st| st.to_string())
                    .collect::<Vec<_>>(),
            )),
            language: Metadata::Language(Some("한국어".to_string())),
            content_type: Metadata::ContentType(Some(ContentType::Manga)),
            created_at: Metadata::CreatedAt(Some("2020-09-02 10:01:00 -05:00".to_string())),
            // content_url: Metadata::ContentURL(None),
            thumbnail_url: Metadata::ThumbnailURL(Some("/bigtn/e/0a/2fd1808fbf15b1901bb6eb751ee88a517bd67ea44061d74f6bd9e4c63ae620ae.jpg".to_string())),
        };

        assert_eq!(expected.id, pd.id);
        assert_eq!(expected.title, pd.title);
        assert_eq!(expected.artists, pd.artists);
        assert_eq!(expected.series, pd.series);
        // assert_eq!(expected.groups, pd.groups);
        // assert_eq!(expected.characters, pd.characters);
        assert_eq!(expected.tags, pd.tags);
        assert_eq!(expected.language, pd.language);
        assert_eq!(expected.content_type, pd.content_type);
        assert_eq!(expected.created_at, pd.created_at);
        assert_eq!(expected.thumbnail_url, pd.thumbnail_url);

        Ok(())
    } */

    #[test]
    fn parse_title() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1399900);

        let gallery_block = gallery_block.request()?;

        let fragment = Html::parse_fragment(&gallery_block.request_data()?);

        let title = gallery_block.parse_metadata(&fragment, Metadata::Title(None));

        let expected = Metadata::Title(Some("COMIC LO 2019-05".to_string()));

        assert_eq!(expected, title);

        Ok(())
    }

    /* #[test]
     fn parse_content_url() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1399900);

        let gallery_block = gallery_block.request()?;

        let fragment = Html::parse_fragment(gallery_block.request_data.unwrap().as_str());

        let content_url = gallery_block.parse_metadata(&fragment, Metadata::ContentURL(None));

        let expected = Metadata::ContentURL(Some(
            "/manga/comic-lo-2019-05-한국어-1399900.html".to_string(),
        ));

        assert_eq!(expected, content_url);

        Ok(())
    } */

    #[test]
    fn parse_thumbnail_url() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1399900);

        let gallery_block = gallery_block.request()?;

        let fragment = Html::parse_fragment(&gallery_block.request_data()?);

        let content_url = gallery_block.parse_metadata(&fragment, Metadata::ThumbnailURL(None));

        let expected = Metadata::ThumbnailURL(Some(
            "/bigtn/2/7b/63c1f20d7bb770faadf60a1a353d64f29c0d51f958bca76cc8e05fb3d19f57b2.jpg"
                .to_string(),
        ));

        assert_eq!(expected, content_url);

        Ok(())
    }

    #[test]
    fn parse_artists() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1399900);

        let gallery_block = gallery_block.request()?;

        let fragment = Html::parse_fragment(&gallery_block.request_data()?);

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
            .map(|st| st.to_string())
            .collect::<Vec<_>>(),
        ));

        assert_eq!(expected, artists);

        Ok(())
    }

    #[test]
    fn parse_artists_is_nothing() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1722267);

        let gallery_block = gallery_block.request()?;

        let fragment = Html::parse_fragment(&gallery_block.request_data()?);

        let artists = gallery_block.parse_metadata(&fragment, Metadata::Artists(None));

        let expected = Metadata::Artists(None);

        assert_eq!(expected, artists);

        Ok(())
    }

    #[test]
    fn parse_language() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1399900);

        let gallery_block = gallery_block.request()?;

        let fragment = Html::parse_fragment(&gallery_block.request_data()?);

        let content_type = gallery_block.parse_metadata(&fragment, Metadata::Language(None));

        let expected = Metadata::Language(Some(Language::Korean));

        assert_eq!(expected, content_type);

        Ok(())
    }

    #[test]
    fn parse_content_type() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1399900);

        let gallery_block = gallery_block.request()?;

        let fragment = Html::parse_fragment(&gallery_block.request_data()?);

        let content_type = gallery_block.parse_metadata(&fragment, Metadata::ContentType(None));

        let expected = Metadata::ContentType(Some(ContentType::Manga));

        assert_eq!(expected, content_type);

        Ok(())
    }

    #[test]
    fn parse_series() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1277807);

        let gallery_block = gallery_block.request()?;

        let fragment = Html::parse_fragment(&gallery_block.request_data()?);

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
            "yuragisou no yuuna-san",
            "yuru camp",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

        let expected = Metadata::Series(Some(e));

        assert_eq!(expected, series);

        Ok(())
    }

    #[test]
    fn parse_series_is_nothing() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1399900);

        let gallery_block = gallery_block.request()?;

        let fragment = Html::parse_fragment(&gallery_block.request_data()?);

        let series_nothing = gallery_block.parse_metadata(&fragment, Metadata::Series(None));

        let expected = Metadata::Series(None);

        assert_eq!(expected, series_nothing);

        Ok(())
    }

    #[test]
    fn parse_tags() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1724122);

        let gallery_block = gallery_block.request()?;

        let fragment = Html::parse_fragment(&gallery_block.request_data()?);

        let tags = gallery_block.parse_metadata(&fragment, Metadata::Tags(None));

        let expected = Metadata::Tags(Some(
            ["footjob ♀", "loli ♀", "sister ♀", "incest"]
                .iter()
                .map(|st| st.to_string())
                .collect::<Vec<_>>(),
        ));

        assert_eq!(expected, tags);

        Ok(())
    }

    #[test]
    fn parse_tags_is_nothing() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1686905);

        let gallery_block = gallery_block.request()?;

        let fragment = Html::parse_fragment(&gallery_block.request_data()?);

        let tags = gallery_block.parse_metadata(&fragment, Metadata::Tags(None));

        let expected = Metadata::Tags(None);

        assert_eq!(expected, tags);

        Ok(())
    }

    #[test]
    fn parse_created_at() -> anyhow::Result<()> {
        let gallery_block = GalleryBlock::new(1724122);

        let gallery_block = gallery_block.request()?;

        let fragment = Html::parse_fragment(&gallery_block.request_data()?);

        let created_at = gallery_block.parse_metadata(&fragment, Metadata::CreatedAt(None));

        // uncomment v2
        // let expected = Metadata::CreatedAt(Some("2020-09-02 10:01:00 -05:00".to_string()));

        let expected = Metadata::CreatedAt(Some("2020-09-02 10:01:00-05".to_string()));

        assert_eq!(expected, created_at);

        Ok(())
    }
}
