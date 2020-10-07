use std::future::Future;
use std::iter::IntoIterator;
use std::pin::Pin;

use anyhow;
use log::{debug, error};
use tokio;
use tokio::sync::mpsc;

fn seperate<T>(it: impl IntoIterator<Item = T>, by: usize) -> Vec<Vec<T>> {
    let by = by - 1;
    let mut r: Vec<Vec<T>> = vec![];
    let mut i = 0;
    for elem in it {
        if let Some(a) = r.get_mut(i) {
            if a.len() >= by {
                i += 1;
            }
            a.push(elem);
        } else {
            r.push(vec![elem]);
        }
    }

    r
}

pub type PinFuture<T> = Pin<Box<dyn Future<Output = Result<T, anyhow::Error>> + Send + 'static>>;

pub async fn await_futures<T: Send + 'static>(
    futures: Vec<
        Pin<Box<impl Future<Output = Result<T, anyhow::Error>> + Send + 'static + ?Sized>>,
    >,
    concurrency_limit: usize,
) -> Result<Vec<T>, anyhow::Error> {
    let (tx, mut rx) = mpsc::channel::<Result<T, anyhow::Error>>(1000);

    let futures_len = futures.len();
    let futures = seperate(futures, concurrency_limit);

    for futs in futures {
        // debug!("//");
        for future in futs {
            let mut tx = tx.clone();
            tokio::spawn(async move {
                let awaited: Result<T, anyhow::Error> = future.await;

                tx.send(awaited).await.unwrap_or_else(|err| {
                    error!("{}", err);
                });
            });
        }
    }

    let mut i = 1;
    let mut r: Vec<T> = vec![];

    while let Some(awaited) = rx.recv().await {
        i += 1;
        match awaited {
            Ok(value) => r.push(value),
            Err(err) => return Err(err),
        }

        if r.len() >= (concurrency_limit * i) || r.len() == futures_len {
            break;
        }
    }

    Ok(r)
}

#[cfg(test)]
mod tests {

    use super::{await_futures, seperate, PinFuture};
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

        let r = await_futures(a, 2).await;

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

        let r = await_futures(a, 2).await;

        let expected = true;

        assert_eq!(expected, r.is_err());

        Ok(())
    }

    #[test]
    fn test_seperate() -> anyhow::Result<()> {
        let a: Vec<i32> = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
        ];

        let r = seperate(a, 7);

        let expected: Vec<Vec<i32>> = vec![
            vec![1, 2, 3, 4, 5, 6, 7],
            vec![8, 9, 10, 11, 12, 13, 14],
            vec![15, 16, 17, 18],
        ];

        assert_eq!(expected, r);

        Ok(())
    }
}
