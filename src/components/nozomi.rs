use std::convert::TryInto;
use std::slice;

use anyhow;
use bytes::Bytes;
use reqwest;

use crate::components::url;

pub struct NozomiParser;

impl NozomiParser {
    pub async fn request(page: usize, per_page: usize) -> anyhow::Result<Bytes> {
        let client = reqwest::Client::builder().build()?;

        let start_bytes = (page - 1) * per_page * 4;
        let end_bytes = start_bytes + per_page * 4 - 1;

        let bytes = client
            .get(url::nozomi().as_str())
            .header("Range", format!("bytes={}-{}", start_bytes, end_bytes))
            .send()
            .await?
            .bytes()
            .await?;

        Ok(bytes)
    }

    pub async fn parse(nozomi: Bytes) -> anyhow::Result<Vec<i32>> {
        let nozomi = nozomi.as_ref();

        let mut res = vec![];

        for i in (0..nozomi.len()).step_by(4) {
            let mut temp: i32 = 0;

            for j in 0..3 {
                temp += TryInto::<i32>::try_into(nozomi[i + (3 - j)]).unwrap() << (j << 3);
            }

            res.push(temp);
        }

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::NozomiParser;

    #[tokio::test]
    async fn parse_nozomi() -> anyhow::Result<()> {
        let nozomi = NozomiParser::request(1, 25).await?;
        let res = NozomiParser::parse(nozomi).await?;

        Ok(())
    }
}
