use anyhow;
use async_trait::async_trait;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::parser::Parser;

pub struct Image {
    id: i32,
}

impl Image {
    pub fn new(id: i32) -> Image {
        Image { id }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    width: i32,
    height: i32,
    hash: String,
    haswebp: Option<u8>,
    hasavifsmalltn: Option<u8>,
    hasavif: Option<u8>,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageInfo {
    files: Vec<FileInfo>,
}

#[async_trait]
impl Parser for Image {
    type RequestData = String;
    type ParseData = Vec<FileInfo>;

    async fn url(&self) -> anyhow::Result<String> {
        Ok(format!("https://ltn.hitomi.la/galleries/{}.js", self.id))
    }

    async fn request(&self) -> anyhow::Result<Self::RequestData> {
        let client = reqwest::Client::builder().build()?;

        let rd = client
            .get(self.url().await?.as_str())
            .send()
            .await?
            .text()
            .await?;

        let rd = rd.split("=").last().unwrap().trim();

        Ok(rd.to_string())
    }

    async fn parse(&self, request_data: Self::RequestData) -> anyhow::Result<Self::ParseData> {
        let image_info = serde_json::from_str::<'_, ImageInfo>(request_data.as_str())?;

        Ok(image_info.files)
    }
}

#[cfg(test)]
mod tests {
    use anyhow;

    use super::Parser;
    use super::Image;

    #[tokio::test]
    async fn parse_image_files_info() -> anyhow::Result<()>{
        let image = Image::new(1721169);

        let rd = image.request().await?;

        let image_files_info = image.parse(rd).await?;

        assert_eq!(10, image_files_info.len());

        Ok(())
    }
}
