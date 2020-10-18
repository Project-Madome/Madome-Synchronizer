pub mod parser;

pub mod utils;

pub mod stage {
    use log::{error, info};
    use madome_client::book::Book;

    pub enum Stage {
        Download,
        Upload,
    }

    pub struct ParseStage;

    impl ParseStage {
        pub fn update(r: &anyhow::Result<Book>) {
            match r {
                Ok(book) => info!("parse finish\nid = {}\ntitle = {}", book.id, book.title),
                Err(err) => error!("parse error\n{:?}", err),
            }
        }
    }

    type DownloadStageInner = (u32, usize, String);
    pub struct DownloadStage;

    impl DownloadStage {
        pub fn update(r: &anyhow::Result<DownloadStageInner>) {
            match r {
                Ok((id, current_page, ext)) => info!(
                    "download finish\nid = {}\npage = {}\next = {}",
                    id, current_page, ext
                ),
                Err(err) => error!("download error\n{:?}", err),
            }
        }
    }

    type URLPath = String;
    type UploadStageInner<'a> = (URLPath, usize);
    pub struct UploadStage;

    impl UploadStage {
        pub fn update(r: &anyhow::Result<UploadStageInner>) {
            match r {
                Ok((url_path, current_page)) => info!(
                    "upload finish\nurl_path = {}\npage = {}",
                    url_path, current_page
                ),
                Err(err) => error!("upload error\n{:?}", err),
            }
        }
    }

    pub struct CompleteStage;

    impl CompleteStage {
        pub fn update(_r: &anyhow::Result<()>) {}
    }
}
