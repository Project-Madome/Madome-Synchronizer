use anyhow;
use async_trait::async_trait;

mod await_futures;
mod flat;
mod seperate;

pub use await_futures::{await_futures, PinFuture};
pub use flat::flat;
pub use seperate::seperate;

#[async_trait]
pub trait FutureUtil {
    type Item;

    async fn await_futures(self) -> anyhow::Result<Vec<Self::Item>>;
}

pub trait VecUtil {
    type Item;

    fn seperate(self, by: usize) -> Vec<Vec<Self::Item>>;
}

pub trait Flat {
    type Item;

    fn flat(self) -> Vec<Self::Item>;
}

#[async_trait]
impl<T: Send + 'static> FutureUtil for Vec<PinFuture<T>> {
    type Item = T;

    async fn await_futures(self) -> anyhow::Result<Vec<Self::Item>> {
        await_futures(self).await
    }
}

impl<T> VecUtil for Vec<T> {
    type Item = T;

    fn seperate(self, by: usize) -> Vec<Vec<Self::Item>> {
        seperate(self, by)
    }
}

impl<T> Flat for Vec<Vec<T>> {
    type Item = T;

    fn flat(self) -> Vec<Self::Item> {
        flat(self)
    }
}
