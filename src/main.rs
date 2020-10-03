extern crate madome_synchronizer;

use anyhow;
use tokio;

use crate::madome_synchronizer::models::{Book, Language};
use crate::madome_synchronizer::parser;
use crate::madome_synchronizer::parser::Parser;

use crate::madome_synchronizer::utils::{await_futures, PinFuture};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let nozomi = parser::Nozomi::new(1, 22, Language::Korean);

    let rd = nozomi.request().await?;
    let content_ids = nozomi.parse(rd).await?;

    let image = parser::Image::new(content_ids[0]);

    let image_rd = image.request().await?;
    let image_pd = image.parse(image_rd).await?;

    println!("{:?}", image_pd);

    /* let mut futures: Vec<PinFuture<Book>> = vec![];

    for content_id in content_ids {
        let f: PinFuture<Book> = Box::pin(async move {
            println!("#{}", content_id);
            let gallery = parser::Gallery::new(content_id);
            let gallery_block = parser::GalleryBlock::new(content_id);

            let gallery_rd = gallery.request();
            let gallery_block_rd = gallery_block.request();

            let gallery_rd = gallery_rd.await?;
            let gallery_block_rd = gallery_block_rd.await?;
            println!("Ready RequestData #{}", content_id);

            let gallery_pd = gallery.parse(gallery_rd);
            let gallery_block_pd = gallery_block.parse(gallery_block_rd);

            let gallery_pd = gallery_pd.await?;
            let gallery_block_pd = gallery_block_pd.await?;
            println!("Ready ParseData #{}", content_id);

            let mut book = gallery_block_pd;

            book.groups = gallery_pd.groups;
            book.characters = gallery_pd.characters;

            Ok(Book::from(book))
        });

        futures.push(f);
    }

    let books = await_futures(futures, 10).await;

    println!("{:?}", books); */

    // let content_id = *pd.iter().last().unwrap();

    /* let mut a: Vec<dyn Future<Output = Book>> = vec![];

    for content_id in content_ids {
        let f: Box<dyn FnOnce() -> dyn Future<Output = anyhow::Result<Book>>> = Box::new(async || {

        });
    } */

    /* {

    } */
    /* {
        println!("Hello, world!");

        let nozomi = parser::Nozomi::new(20, 100000, "korean".to_string());

        let rd = nozomi.request().await?;
        let pd = nozomi.parse(rd).await?;

        // pd.sort_by(|a, b| b.partial_cmp(a).unwrap());

        println!("Book IDs = {:?}", pd);
        println!("Book Lengths = {}", pd.len());
    } */

    Ok(())
}
