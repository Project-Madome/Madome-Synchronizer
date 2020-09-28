extern crate madome_synchronizer;

use anyhow;

use crate::madome_synchronizer::parser::Parser;
use crate::madome_synchronizer::parser::Nozomi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let nozomi = Nozomi::new(1, 25, String::from("korean"));
    let rd = nozomi.request().await?;
    let pd = nozomi.parse(rd).await?;

    println!("Book IDs = {:?}", pd);

    println!("Hello, world!");

    Ok(())
}
