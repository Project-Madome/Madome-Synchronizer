use anyhow;

mod gallery;
mod gallery_block;
mod image;
mod nozomi;

pub use gallery::Gallery;
pub use gallery_block::GalleryBlock;
pub use image::{File, Image};
pub use nozomi::Nozomi;

pub trait Parser {
    // self.request_data;
    type RequestData;
    type ParseData;

    fn request_data(&self) -> anyhow::Result<&Box<Self::RequestData>>;

    fn url(&self) -> anyhow::Result<String>;

    fn request(self) -> anyhow::Result<Box<Self>>;

    fn parse(&self) -> anyhow::Result<Self::ParseData>;
}
