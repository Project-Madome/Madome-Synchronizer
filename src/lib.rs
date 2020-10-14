pub mod parser;

pub mod utils;

pub mod stage {
    use log::{debug, error};

    pub enum Stage {
        Download,
        Upload,
    }

    type DownloadStageInner = (u32, usize, String);
    pub struct DownloadStage;

    impl DownloadStage {
        pub fn update(r: &anyhow::Result<DownloadStageInner>) {
            match r {
                Ok((id, current_page, ext)) => debug!(
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
                Ok((url_path, current_page)) => debug!(
                    "upload finish\nurl_path = {}\npage = {}",
                    url_path, current_page
                ),
                Err(err) => error!("upload error\n{:?}", err),
            }
        }
    }
}
