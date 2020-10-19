pub mod parser;

pub mod utils;

pub mod stage {

    use log::{error, info};

    pub fn update(id: u32, stage: Stage) -> anyhow::Result<()> {
        match stage {
            Stage::Fail => error!("{}: {}", id, stage.to_string()),
            other => info!("{}: {}", id, other.to_string()),
        }

        Ok(())
    }

    pub enum Stage {
        ParsedBook,
        ParsedImages,
        AddedThumbnail,
        AddedImages,
        AddedBook,
        Fail,
    }

    impl ToString for Stage {
        fn to_string(&self) -> String {
            let r = match self {
                Self::ParsedBook => "Parsed Book",
                Self::ParsedImages => "Parsed Images",
                Self::AddedThumbnail => "Added Thumnail",
                Self::AddedImages => "Added Images",
                Self::AddedBook => "Added Book",
                Self::Fail => "Fail",
            };

            r.to_string()
        }
    }
}
