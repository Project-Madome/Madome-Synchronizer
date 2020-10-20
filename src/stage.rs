use log::{error, info};

pub fn update(id: u32, stage: Stage) -> anyhow::Result<()> {
    match stage {
        Stage::Fail(err) => error!("{}: {}: {:#?}", id, stage.to_string(), err),
        other => info!("{}: {}", id, other.to_string()),
    }

    Ok(())
}

pub enum Stage<'a> {
    ParsedBook,
    ParsedImages,
    AddedThumbnail,
    AddedImages,
    AddedBook,
    Fail(&'a anyhow::Error),
}

impl<'a> ToString for Stage<'a> {
    fn to_string(&self) -> String {
        let r = match self {
            Self::ParsedBook => "Parsed Book",
            Self::ParsedImages => "Parsed Images",
            Self::AddedThumbnail => "Added Thumnail",
            Self::AddedImages => "Added Images",
            Self::AddedBook => "Added Book",
            Self::Fail(_) => "Fail",
        };

        r.to_string()
    }
}
