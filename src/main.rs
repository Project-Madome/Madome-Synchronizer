extern crate madome_synchronizer;

use std::fs;
use std::future::Future;
use std::pin::Pin;

use anyhow;
use bytes::Bytes;
use env_logger;
use futures::stream::{self, StreamExt};
use log::{debug, error, info, trace, warn};
use tokio;

use crate::madome_synchronizer::models::{Book, Language, MetadataBook};
use crate::madome_synchronizer::parser;
use crate::madome_synchronizer::parser::Parser;

use crate::madome_synchronizer::utils::{FutureUtil, PinFuture};

fn init_logger() {
    env_logger::init()
}

async fn fetch_books(content_ids: Vec<i32>) -> anyhow::Result<Vec<Book>> {
    content_ids
        .into_iter()
        .map(|content_id| {
            Box::pin(async move {
                debug!("Content ID #{}", content_id);

                let (gallery_parser, gallery_block_parser): (
                    Box<parser::Gallery>,
                    Box<parser::GalleryBlock>,
                ) = tokio::try_join!(
                    parser::Gallery::new(content_id).request(),
                    parser::GalleryBlock::new(content_id).request()
                )?;
                debug!("Ready RequestData #{}", content_id);

                let (gallery_data, mut gallery_block_data): (MetadataBook, MetadataBook) =
                    tokio::try_join!(gallery_parser.parse(), gallery_block_parser.parse(),)?;
                debug!("Ready ParseData #{}", content_id);

                gallery_block_data.groups = gallery_data.groups;
                gallery_block_data.characters = gallery_data.characters;

                Ok(Book::from(gallery_block_data))
            }) as PinFuture<Book>
        })
        .collect::<Vec<PinFuture<Book>>>()
        .await_futures(25)
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger();

    let nozomi_parser = parser::Nozomi::new(10, 1, Language::Korean);

    let nozomi_parser = nozomi_parser.request().await?;
    let content_ids = nozomi_parser.parse().await?;

    debug!("Content IDs {:?}", content_ids);

    let books = fetch_books(content_ids.clone()).await?;

    debug!("Books {:?}", books);

    /* let image_parser = parser::Image::new(content_ids[0]);
    let image_files = image_parser.request().await?.parse().await?;

    for image_file in image_files {
        let a = image_file.download(content_ids[0]).await?;
        break;
    } */

    let image_files: Vec<(i32, Vec<parser::File>)> = content_ids
        .into_iter()
        .map(|content_id| {
            Box::pin(async move {
                let image_parser = parser::Image::new(content_id);
                let image_files = image_parser.request().await?.parse().await?;

                Ok((content_id, image_files))
            }) as PinFuture<(i32, Vec<parser::File>)>
        })
        .collect::<Vec<_>>()
        .await_futures(25)
        .await?;

    if let Err(_) = fs::create_dir("./.temp") {}

    for (content_id, image_files) in image_files {
        debug!("image length {}", image_files.len());

        image_files
            .into_iter()
            .map(|image_file| {
                Box::pin(async move {
                    let image_bytes = image_file.download(content_id).await?;
                    // TODO: upload code
                    fs::write(format!("./.temp/{}", image_file.name), image_bytes)?;
                    debug!("Image Name {}", image_file.name);

                    Ok(())
                }) as PinFuture<()>
            })
            .collect::<Vec<_>>()
            .await_futures(10)
            .await?;
    }

    // let books = iter(books);

    // debug!("{:?}", r.unwrap());

    /* let image = parser::Image::new(content_ids[0]);

    let image_rd = image.request().await?;
    let image_pd = image.parse(image_rd).await?;

    debug!("{:?}", image_pd[0]);

    let image_url = image_pd[0].url(content_ids[0])?;

    debug!("{}", image_url);

    let image_bytes = image_pd[0].download(content_ids[0]).await?;

    debug!("{}", image_pd[0].name);

    fs::write(format!("./.temp/{}", image_pd[0].name), image_bytes)?; */

    // let content_id = *pd.iter().last().unwrap();

    /* let mut a: Vec<dyn Future<Output = Book>> = vec![];

    for content_id in content_ids {
        let f: Box<dyn FnOnce() -> dyn Future<Output = anyhow::Result<Book>>> = Box::new(async || {

        });
    } */

    /* {

    } */
    /* {
        info!("Hello, world!");

        let nozomi = parser::Nozomi::new(20, 100000, "korean".to_string());

        let rd = nozomi.request().await?;
        let pd = nozomi.parse(rd).await?;

        // pd.sort_by(|a, b| b.partial_cmp(a).unwrap());

        info!("Book IDs = {:?}", pd);
        info!("Book Lengths = {}", pd.len());
    } */

    Ok(())
}
