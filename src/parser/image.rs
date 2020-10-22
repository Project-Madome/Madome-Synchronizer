use std::char;

use anyhow;
use bytes::Bytes;
use log::{debug, trace};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::parser::Parser;

pub struct Image {
    id: u32,
    request_data: Option<Box<String>>,
}

impl Image {
    pub fn new(id: u32) -> Image {
        Image {
            id,
            request_data: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub width: u32,
    pub height: u32,
    pub hash: String,
    pub haswebp: Option<u8>,
    pub hasavifsmalltn: Option<u8>,
    pub hasavif: Option<u8>,
    pub name: String,
}

pub type ImageURL = String;
pub type ThumbnailURL = String;

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

    /* pub fn ext(&self) -> &str {
        Path::new(self.name.as_str())
            .extension()
            .unwrap_or(OsStr::new("img"))
            .to_str()
            .expect("Can't get str from OsStr::to_str()")
    } */

    pub fn url(&self, content_id: u32) -> anyhow::Result<(ImageURL, ThumbnailURL)> {
        trace!("File::url()");
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
        let mut subdomain = if self.has_webp() {
            char::from_u32(97 + c % number_of_frontends)
                .unwrap()
                .to_string()
        } else {
            "b".to_string()
        };

        debug!("1st subdomain {}", subdomain);

        let postfix = &self.hash[self.hash.len() - 3..].chars().collect::<Vec<_>>();

        debug!("hash {}", self.hash);
        debug!("postfix {:?}", postfix);

        let x = format!("{}{}", postfix[0], postfix[1]);

        debug!("x {}", x);

        if let Ok(mut x) = u32::from_str_radix(x.as_str(), 16) {
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

        /* let image_url = if self.has_webp() == false {
            format!(
                "https://{}b.hitomi.la/images/{}/{}{}/{}.{}",
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
        }; */

        let image_url = format!(
            "https://{}b.hitomi.la/images/{}/{}{}/{}.{}",
            subdomain,
            postfix[2],
            postfix[0],
            postfix[1],
            self.hash,
            self.name.split(".").last().unwrap()
        );

        let thumbnail_url = format!(
            "https://tn.hitomi.la/bigtn/{}/{}{}/{}.jpg",
            postfix[2], postfix[0], postfix[1], self.hash
        );

        debug!("image_url = {}", image_url);
        debug!("thumbnail_url = {}", thumbnail_url);

        Ok((image_url, thumbnail_url))
    }

    /// (URL, buf)
    pub fn download(&self, content_id: u32, is_thumbnail: bool) -> anyhow::Result<(String, Bytes)> {
        let (image_url, thumbnail_url) = self.url(content_id)?;

        if is_thumbnail {
            let r = self.download_(content_id, &thumbnail_url)?;
            Ok((thumbnail_url, r))
        } else {
            let r = self.download_(content_id, &image_url)?;
            Ok((image_url, r))
        }
    }

    fn download_<U: reqwest::IntoUrl>(&self, content_id: u32, url: U) -> anyhow::Result<Bytes> {
        trace!("File::download()");
        let client = reqwest::blocking::Client::builder().build()?;

        let response = client
            .get(url)
            .header(
                "Referer",
                format!("https://hitomi.la/reader/{}.html", content_id),
            )
            .send()?;

        if response.status().is_success() {
            let bytes = response.bytes()?;
            Ok(bytes)
        } else {
            // debug!("{}", response.text()?);

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

impl Parser for Image {
    type RequestData = String;
    type ParseData = Vec<File>;

    fn request_data(&self) -> anyhow::Result<&Box<Self::RequestData>> {
        trace!("Image::request_data()");
        match self.request_data {
            Some(ref rd) => Ok(rd),
            None => Err(anyhow::Error::msg("Can't get request_data")),
        }
    }

    fn url(&self) -> anyhow::Result<String> {
        trace!("Image::url()");
        Ok(format!("https://ltn.hitomi.la/galleries/{}.js", self.id))
    }

    fn request(mut self) -> anyhow::Result<Box<Self>> {
        trace!("Image::request()");
        let client = reqwest::blocking::Client::builder().build()?;

        let response = client.get(self.url()?.as_str()).send()?;

        if !response.status().is_success() {
            return Err(anyhow::Error::msg(response.status().to_string()));
        }

        let rd = response.text()?;

        // panic
        let i = rd.find("=").ok_or_else(|| {
            anyhow::Error::msg(format!(
                "error occurs `request_data.find(\"=\")` in parser::Image::request(), {}",
                rd
            ))
        })?;
        let rd = &rd[i + 1..];

        self.request_data = Some(Box::new(rd.to_string()));
        Ok(Box::new(self))
    }

    fn parse(&self) -> anyhow::Result<Self::ParseData> {
        trace!("Image::parse()");
        let ref request_data = match self.request_data {
            Some(ref rd) => rd,
            None => return Err(anyhow::Error::msg("Can't get request_data")),
        };

        let image_info = serde_json::from_str::<'_, ImageInfo>(request_data.as_str())?;

        let files = image_info.files;

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use anyhow;

    use super::Image;
    use super::Parser;

    #[test]
    fn parse_image_files_info() -> anyhow::Result<()> {
        let image_parser = Image::new(1721169);

        let image_parser = image_parser.request()?;

        let image_files_info = image_parser.parse()?;

        assert_eq!(10, image_files_info.len());

        Ok(())
    }
}
