extern crate madome_synchronizer;

use anyhow;

use crate::madome_synchronizer::components::nozomi::NozomiParser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let nozomi = NozomiParser::request(1, 25).await?;
    let nozomi = NozomiParser::parse(nozomi).await?;

    println!("Book IDs = {:?}", nozomi);

    println!("Hello, world!");

    Ok(())
}
