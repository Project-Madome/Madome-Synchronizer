extern crate madome_synchronizer;

use anyhow;

use crate::madome_synchronizer::parser;
use crate::madome_synchronizer::parser::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let nozomi = parser::Nozomi::new(1, 25, "korean".to_string());

    let rd = nozomi.request().await?;
    let pd = nozomi.parse(rd).await?;

    println!("Book IDs = {:?}", pd);

    {
        println!("-----------------------------------");
        println!("Gallery");

        let gallery = parser::Gallery::new(pd[13]);

        let rd = gallery.request().await?;

        println!("{}", rd);
    }

    {
        println!("-----------------------------------");
        println!("GalleryBlock");

        let gallery_block = parser::GalleryBlock::new(pd[13]);

        let rd = gallery_block.request().await?;

        println!("{}", rd);

        let pd = gallery_block.parse(rd).await?;
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
