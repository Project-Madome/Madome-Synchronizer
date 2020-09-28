use std::convert::TryInto;

use anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use reqwest;

use super::Parser;

pub struct Nozomi {
    page: usize,
    per_page: usize,
    language: String,
}

impl Nozomi {
    pub fn new(page: usize, per_page: usize, language: String) -> Nozomi {
        Nozomi {
            page,
            per_page,
            language,
        }
    }
}

#[async_trait]
impl Parser for Nozomi {
    type RequestData = Bytes;
    type ParseData = Vec<i32>;

    fn url(&self) -> String {
        format!(
            "https://ltn.hitomi.la/index-{}.nozomi",
            self.language.to_lowercase()
        )
    }

    async fn request(&self) -> anyhow::Result<Self::RequestData> {
        let client = reqwest::Client::builder().build()?;

        let start_bytes = (self.page - 1) * self.per_page * 4;
        let end_bytes = start_bytes + self.per_page * 4 - 1;

        let bytes = client
            .get(self.url().as_str())
            .header("Range", format!("bytes={}-{}", start_bytes, end_bytes))
            .send()
            .await?
            .bytes()
            .await?;

        Ok(bytes)
    }

    async fn parse(&self, nozomi: Bytes) -> anyhow::Result<Self::ParseData> {
        let nozomi = nozomi.as_ref();

        let mut res = vec![];

        for i in (0..nozomi.len()).step_by(4) {
            let mut temp: i32 = 0;

            for j in 0..3 {
                temp += TryInto::<i32>::try_into(nozomi[i + (3 - j)]).unwrap() << (j << 3);
            }

            res.push(temp);
        }

        res.sort_by(|a, b| b.partial_cmp(a).unwrap());

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::Parser;
    use super::Nozomi;

    #[tokio::test]
    async fn parse_nozomi() -> anyhow::Result<()> {
        let nozomi = Nozomi::new(1, 25, String::from("korean"));

        let rd = nozomi.request().await?;

        let pd = nozomi.parse(rd).await?;

        assert_eq!(25, pd.len());

        Ok(())
    }
}
