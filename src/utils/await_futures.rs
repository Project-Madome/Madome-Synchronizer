use std::future::Future;
use std::pin::Pin;

use anyhow;
use log::error;
use tokio;
use tokio::sync::mpsc;

pub type PinFuture<T> = Pin<Box<dyn Future<Output = Result<T, anyhow::Error>> + Send + 'static>>;

pub async fn await_futures<T: Send + 'static>(
    futures: Vec<
        Pin<Box<impl Future<Output = Result<T, anyhow::Error>> + Send + 'static + ?Sized>>,
    >,
) -> Result<Vec<T>, anyhow::Error> {
    let (tx, mut rx) = mpsc::channel::<Result<T, anyhow::Error>>(1000);

    let futures_len = futures.len();

    let mut threads: Vec<tokio::task::JoinHandle<()>> = vec![];

    // debug!("//");
    for future in futures {
        let mut tx = tx.clone();
        let thread = tokio::spawn(async move {
            let awaited: Result<T, anyhow::Error> = future.await;

            tx.send(awaited).await.unwrap_or_else(|err| {
                error!("{}", err);
            });
        });

        threads.push(thread);
    }

    let mut r: Vec<T> = vec![];

    while let Some(awaited) = rx.recv().await {
        match awaited {
            Ok(value) => r.push(value),
            Err(err) => {
                for thread in threads {
                    let _ = thread.await;
                }
                return Err(err);
            }
        }

        if r.len() == futures_len {
            break;
        }
    }

    Ok(r)
}

#[cfg(test)]
mod tests {

    use super::{await_futures, PinFuture};
    use anyhow;

    async fn af(i: i32) -> anyhow::Result<i32> {
        Ok(i)
    }

    async fn bf(_: i32) -> anyhow::Result<i32> {
        Err(anyhow::Error::msg("failed"))
    }

    #[tokio::test]
    async fn test_await_futures() -> anyhow::Result<()> {
        let a: Vec<PinFuture<i32>> = vec![
            Box::pin(af(1)),
            Box::pin(af(2)),
            Box::pin(af(3)),
            Box::pin(af(4)),
            Box::pin(af(5)),
        ];

        let r = await_futures(a).await;

        let expected = vec![1, 2, 3, 4, 5];

        assert_eq!(expected, r.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_await_futures_failed() -> anyhow::Result<()> {
        let a: Vec<PinFuture<i32>> = vec![
            Box::pin(af(1)),
            Box::pin(af(2)),
            Box::pin(bf(3)),
            Box::pin(af(4)),
            Box::pin(af(5)),
        ];

        let r = await_futures(a).await;

        let expected = true;

        assert_eq!(expected, r.is_err());

        Ok(())
    }
}
