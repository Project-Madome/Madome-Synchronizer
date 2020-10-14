use std::convert::TryInto;

use anyhow;
use bytes::Bytes;
use log::{debug, trace};
use madome_client::book::Language;
use reqwest;

use super::Parser;

/// # Nozomi Parser
/// Not needed VPN for Nozomi Parser
///
/// ##
pub struct Nozomi {
    page: usize,
    per_page: usize,
    language: String,
    request_data: Option<Box<Bytes>>,
}

impl Nozomi {
    pub fn new(page: usize, per_page: usize, language: Language) -> Nozomi {
        Nozomi {
            page,
            per_page,
            language: language.into(),
            request_data: None,
        }
    }
}

impl Parser for Nozomi {
    type RequestData = Bytes;
    type ParseData = Vec<u32>;

    fn request_data(&self) -> anyhow::Result<&Box<Self::RequestData>> {
        match self.request_data {
            Some(ref rd) => Ok(rd),
            None => Err(anyhow::Error::msg("Can't get request_data")),
        }
    }

    fn url(&self) -> anyhow::Result<String> {
        Ok(format!(
            "https://ltn.hitomi.la/index-{}.nozomi",
            self.language.to_lowercase()
        ))
    }

    fn request(mut self) -> anyhow::Result<Box<Self>> {
        trace!("Nozomi::request()");
        let client = reqwest::blocking::Client::builder().build()?;

        let start_bytes = (self.page - 1) * self.per_page * 4;
        let end_bytes = start_bytes + self.per_page * 4 - 1;

        debug!("start_bytes = {}", start_bytes);
        debug!("end_bytes = {}", end_bytes);

        let bytes = client
            .get(self.url()?.as_str())
            .header("Range", format!("bytes={}-{}", start_bytes, end_bytes))
            .send()?
            .bytes()?;

        self.request_data = Some(Box::new(bytes));
        Ok(Box::new(self))
    }

    fn parse(&self) -> anyhow::Result<Self::ParseData> {
        trace!("Nozomi::parse()");
        let request_data = self.request_data()?;

        let mut res = vec![];

        'a: for i in (0..request_data.len()).step_by(4) {
            let mut temp: u32 = 0;

            for j in 0..3 {
                // https://github.com/Project-Madome/Madome-Synchronizer/issues/1
                // temp += TryInto::<i32>::try_into(request_data[i + (3 - j)])? << (j << 3);
                if let Some(a) = request_data.get(i + (3 - j)) {
                    temp += TryInto::<u32>::try_into(*a)? << (j << 3);
                } else {
                    break 'a;
                }
            }

            debug!("id = {}", temp);

            res.push(temp);
        }

        res.sort_by(|a, b| b.cmp(a));

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use madome_client::book::Language;

    use super::Nozomi;
    use super::Parser;

    #[test]
    fn parse_nozomi() -> anyhow::Result<()> {
        let nozomi_parser = Nozomi::new(1, 25, Language::Korean);

        let nozomi_parser = nozomi_parser.request()?;

        let pd = nozomi_parser.parse()?;

        assert_eq!(25, pd.len());

        Ok(())
    }

    #[test]
    fn parse_nozomi_index_out_of_bounds() -> anyhow::Result<()> {
        let nozomi_parser = Nozomi::new(20, 1000000, Language::Korean);

        let nozomi_parser = nozomi_parser.request()?;
        let pd = nozomi_parser.parse()?;

        Ok(())
    }
}
