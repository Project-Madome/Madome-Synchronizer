use std::char;
use std::error;

use anyhow;
use async_trait::async_trait;
use bytes::Bytes;
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
pub struct File {
    pub width: i32,
    pub height: i32,
    pub hash: String,
    pub haswebp: Option<u8>,
    pub hasavifsmalltn: Option<u8>,
    pub hasavif: Option<u8>,
    pub name: String,
}

impl File {
    pub fn has_webp(&self) -> bool {
        if let Some(haswebp) = self.haswebp {
            if haswebp == 0 {
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn url(&self, id: i32) -> anyhow::Result<String> {
        let id_string = id.to_string();
        let mut id_chars = id_string.chars();

        let c: u32 = id_chars
            .nth(id_string.len() - 1)
            .unwrap()
            .to_digit(16)
            .expect(format!("Can't id_char.to_digit(16); id = {}", id).as_str());

        let number_of_frontends = 3;
        let mut subdomain = char::from_u32(97 + c % number_of_frontends)
            .unwrap()
            .to_string();

        let postfix = &self.hash[self.hash.len() - 3..].chars().collect::<Vec<_>>();

        let x = format!("{}{}", postfix[0], postfix[1]);
        let x = i32::from_str_radix(x.as_str(), 16);
        let x_is_ok = x.is_ok();
        let x = x.unwrap_or(0);

        if x_is_ok {
            let n = if x < 0x30 { 2 } else { 3 };

            subdomain = char::from_u32(97 + (x % n) as u32).unwrap().to_string();
        }

        let r = if self.has_webp() == false {
            format!(
                "https://{}a.hitomi.la/images/{}/{}{}/{}.{}",
                subdomain,
                postfix[2],
                postfix[0],
                postfix[1],
                self.hash,
                self.name.split(".").last().unwrap()
            )
        } else if self.hash.as_str() == "" {
            format!("https://{}a.hitomi.la/webp/{}.webp", subdomain, self.name)
        } else if self.hash.len() < 3 {
            format!("https://{}a.hitomi.la/webp/{}.webp", subdomain, self.hash)
        } else {
            format!(
                "https://{}a.hitomi.la/webp/{}/{}{}/{}.webp",
                subdomain, postfix[2], postfix[0], postfix[1], self.hash
            )
        };

        Ok(r)
    }

    pub async fn download(&self, id: i32) -> anyhow::Result<Bytes> {
        let client = reqwest::Client::builder().build()?;

        let bytes = client.get(self.url(id)?.as_str()).header("Referer", format!("https://hitomi.la/galleries/{}.html", id)).send().await?.bytes().await?;

        Ok(bytes)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageInfo {
    files: Vec<File>,
}

#[async_trait]
impl Parser for Image {
    type RequestData = String;
    type ParseData = Vec<File>;

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

        let rd = &rd[rd.find("=").unwrap() + 1..];

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

    use super::Image;
    use super::Parser;

    #[tokio::test]
    async fn parse_image_files_info() -> anyhow::Result<()> {
        let image = Image::new(1721169);

        let rd = image.request().await?;

        let image_files_info = image.parse(rd).await?;

        assert_eq!(10, image_files_info.len());

        Ok(())
    }
}
