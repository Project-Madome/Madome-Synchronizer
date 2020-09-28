use anyhow;
use async_trait::async_trait;

mod nozomi;

pub use nozomi::Nozomi;
pub mod url;

#[async_trait]
pub trait Parser {
    type RequestData;
    type ParseData;

    fn url(&self) -> String;

    async fn request(&self) -> anyhow::Result<Self::RequestData>;

    async fn parse(&self, request_data: Self::RequestData) -> anyhow::Result<Self::ParseData>;
}
