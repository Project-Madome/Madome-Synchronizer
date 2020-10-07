use anyhow;
use async_trait::async_trait;

mod await_futures;

pub use await_futures::{await_futures, PinFuture};

#[async_trait]
pub trait FutureUtil {
    type Item;

    async fn await_futures(self, concurrency_limit: usize) -> anyhow::Result<Vec<Self::Item>>;
}

#[async_trait]
impl<T: Send + 'static> FutureUtil for Vec<PinFuture<T>> {
    type Item = T;

    async fn await_futures(self, concurrency_limit: usize) -> anyhow::Result<Vec<Self::Item>> {
        await_futures(self, concurrency_limit).await
    }
}
