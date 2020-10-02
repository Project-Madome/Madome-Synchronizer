extern crate madome_synchronizer;

use anyhow;

use crate::madome_synchronizer::models::Book;
use crate::madome_synchronizer::parser;
use crate::madome_synchronizer::parser::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let nozomi = parser::Nozomi::new(30000, 25, "korean".to_string());

    let rd = nozomi.request().await?;
    let pd = nozomi.parse(rd).await?;

    println!("Book IDs = {:?}", pd);

    let content_id = *pd.iter().last().unwrap();

    {
        println!("-----------------------------------");
        println!("Gallery");

        let gallery = parser::Gallery::new(content_id);

        let gallery_rd = gallery.request().await?;

        println!("{}", gallery_rd);

        let gallery_pd = gallery.parse(gallery_rd).await?;

        println!("-----------------------------------");
        println!("GalleryBlock");

        let gallery_block = parser::GalleryBlock::new(content_id);

        let gallery_block_rd = gallery_block.request().await?;

        println!("{}", gallery_block_rd);

        let mut gallery_block_pd = gallery_block.parse(gallery_block_rd).await?;

        gallery_block_pd.characters = gallery_pd.characters;
        gallery_block_pd.groups = gallery_pd.groups;

        let book: Book = gallery_block_pd.into();

        println!("book = {:?}", book);
    }

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
