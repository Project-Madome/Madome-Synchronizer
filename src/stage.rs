use log::{error, info};

/* pub fn update(id: u32, stage: Stage) -> anyhow::Result<()> {
    match stage {
        Stage::Fail(err) => error!("{}: {}: {:#?}", id, stage.to_string(), err),
        other => info!("{}: {}", id, other.to_string()),
    }

    Ok(())
} */

pub fn update<ID, T, F>(id: ID, stage: Stage, f: F) -> anyhow::Result<T>
where
    ID: std::fmt::Display,
    F: Fn() -> StageR<T>,
{
    // Add Images Ready 여러번 출력됨
    // 해결할  방법
    info!("{}: {}: {}", id, stage, State::Ready);

    let StageR(state, progress, r) = f();

    match r {
        Ok(r) => {
            if let Some(progress) = progress {
                if 100.0 <= progress {
                    info!("{}: {}: {}: {}%", id, stage, State::Fulfilled, progress);
                } else {
                    info!("{}: {}: {}: {}%", id, stage, state, progress);
                }
            } else {
                info!("{}: {}: {}", id, stage, state);
            }
            Ok(r)
        }
        Err(err) => {
            error!("{}: {}: Error: {:#?}", id, stage, err);
            Err(err)
        }
    }
}

pub type Progress = f64;

pub struct StageR<T>(pub State, pub Option<Progress>, pub anyhow::Result<T>);

pub enum State {
    Ready,
    Pending,
    Fulfilled,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            Self::Ready => "Ready",
            Self::Pending => "Pending",
            Self::Fulfilled => "Fulfilled",
        };

        write!(f, "{}", r)
    }
}

pub enum Stage {
    ParseBook,
    ParseImages,
    AddThumbnail,
    AddImages,
    AddImageList,
    AddBook,
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            Self::ParseBook => "Parse Book",
            Self::ParseImages => "Parse Images",
            Self::AddThumbnail => "Add Thumnail",
            Self::AddImages => "Add Images",
            Self::AddImageList => "Add Image List",
            Self::AddBook => "Add Book",
        };

        write!(f, "{}", r)
    }
}
