use anyhow;
use html5ever::parse_document;
use reqwest;

use crate::parser::url;

pub struct Galleries;

impl Galleries {
    pub async fn request_url(id: i32) -> anyhow::Result<String> {
        let client = reqwest::Client::builder().build()?;

        let galleries_html = client
            .get(url::galleries(id).as_str())
            .send()
            .await?
            .text()
            .await?;

        Ok(galleries_html)
    }

    pub async fn parse_url(id: i32) -> anyhow::Result<()> {
        let galleries_html = GalleriesParser::request_url(id).await?;

        println!("{}", galleries_html);

        Ok(())
    }
}
