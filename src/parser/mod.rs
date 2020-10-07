use anyhow;
use async_trait::async_trait;

mod gallery;
mod gallery_block;
mod image;
mod nozomi;

pub use gallery::Gallery;
pub use gallery_block::GalleryBlock;
pub use image::{File, Image};
pub use nozomi::Nozomi;

#[async_trait]
pub trait Parser {
    // self.request_data;
    type RequestData;
    type ParseData;

    fn request_data(&self) -> anyhow::Result<&Box<Self::RequestData>>;

    async fn url(&self) -> anyhow::Result<String>;

    async fn request(mut self) -> anyhow::Result<Box<Self>>;

    async fn parse(&self) -> anyhow::Result<Self::ParseData>;
}
