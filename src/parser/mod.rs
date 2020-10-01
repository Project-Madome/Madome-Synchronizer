use anyhow;
use async_trait::async_trait;

// mod galleries;
mod gallery_block;
mod nozomi;

// pub use galleries::Galleries;
pub use gallery_block::GalleryBlock;
pub use nozomi::Nozomi;

#[async_trait]
pub trait Parser {
    type RequestData;
    type ParseData;

    fn url(&self) -> String;

    async fn request(&self) -> anyhow::Result<Self::RequestData>;

    async fn parse(&self, request_data: Self::RequestData) -> anyhow::Result<Self::ParseData>;
}
