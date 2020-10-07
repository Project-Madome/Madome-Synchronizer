use std::char;

use anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use log::debug;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::parser::Parser;

pub struct Image {
    id: i32,
    request_data: Option<Box<String>>,
}

impl Image {
    pub fn new(id: i32) -> Image {
        Image {
            id,
            request_data: None,
        }
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

    pub fn url(&self, content_id: i32) -> anyhow::Result<String> {
        let id_string = content_id.to_string();
        let mut id_chars = id_string.chars();

        let c: u32 = id_chars
            .nth(id_string.len() - 1)
            .unwrap()
            .to_string()
            .encode_utf16()
            .last()
            .unwrap()
            .into();

        debug!("id_char utf16 code {}", c);

        let number_of_frontends = 3;
        let mut subdomain = char::from_u32(97 + c % number_of_frontends)
            .unwrap()
            .to_string();

        debug!("1st subdomain {}", subdomain);

        let postfix = &self.hash[self.hash.len() - 3..].chars().collect::<Vec<_>>();

        debug!("hash {}", self.hash);
        debug!("postfix {:?}", postfix);

        let x = format!("{}{}", postfix[0], postfix[1]);

        debug!("x {}", x);

        if let Ok(mut x) =u32::from_str_radix(x.as_str(), 16) {
            let mut n: u32 = 3;

            debug!("x {}", x);
            if x < 0x30 {
                n = 2;
            }
            if x < 0x09 {
                x = 1;
            }
            debug!("n {}", n);

            subdomain = char::from_u32(97 + (x % n)).unwrap().to_string();
        }

        debug!("2nd subdomain {}", subdomain);

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

        debug!("image_url {}", r);

        Ok(r)
    }

    pub async fn download(&self, content_id: i32) -> anyhow::Result<Bytes> {
        let client = reqwest::Client::builder().build()?;

        let response = client
            .get(self.url(content_id)?.as_str())
            .header(
                "Referer",
                format!("https://hitomi.la/reader/{}.html", content_id),
            )
            .send()
            .await?;

        if response.status().is_success() {
            let bytes = response.bytes().await?;
            Ok(bytes)
        } else {
            // debug!("{}", response.text().await?);

            Err(anyhow::Error::msg(format!(
                "Image Download Error! {}",
                response.status().to_string()
            )))
        }
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

    fn request_data(&self) -> anyhow::Result<&Box<Self::RequestData>> {
        match self.request_data {
            Some(ref rd) => Ok(rd),
            None => Err(anyhow::Error::msg("Can't get request_data")),
        }
    }

    async fn url(&self) -> anyhow::Result<String> {
        Ok(format!("https://ltn.hitomi.la/galleries/{}.js", self.id))
    }

    async fn request(mut self) -> anyhow::Result<Box<Self>> {
        let client = reqwest::Client::builder().build()?;

        let rd = client
            .get(self.url().await?.as_str())
            .send()
            .await?
            .text()
            .await?;

        let rd = &rd[rd.find("=").unwrap() + 1..];

        self.request_data = Some(Box::new(rd.to_string()));
        Ok(Box::new(self))
    }

    async fn parse(&self) -> anyhow::Result<Self::ParseData> {
        let ref request_data = match self.request_data {
            Some(ref rd) => rd,
            None => return Err(anyhow::Error::msg("Can't get request_data")),
        };

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
        let image_parser = Image::new(1721169);

        let image_parser = image_parser.request().await?;

        let image_files_info = image_parser.parse().await?;

        assert_eq!(10, image_files_info.len());

        Ok(())
    }
}
