use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::sync::Mutex;

use log::{error, info};

pub struct StageUpdater<ID>
where
    ID: Display,
{
    id: ID,
    inner: Mutex<HashMap<u8, usize>>,
}

impl<ID> StageUpdater<ID>
where
    ID: Display,
{
    pub fn new(id: ID) -> Self {
        Self {
            id,
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn update<T, F>(&self, stage: Stage, f: F) -> anyhow::Result<T>
    where
        F: Fn() -> StageR<T>,
    {
        {
            let mut inner = self.inner.lock().unwrap();

            if !inner.contains_key(&stage.as_u8()) {
                inner.insert(stage.as_u8(), 0);
                info!("{}: {}: {}", self.id, stage, State::Ready);
            }
        }

        let StageR(state, max_call_count, r) = f();

        let current_call_count: usize = {
            let mut inner = self.inner.lock().unwrap();

            let count = inner.get_mut(&stage.as_u8()).unwrap();
            *count += 1;
            *count
        };

        match r {
            Ok(r) => {
                if let Some(max_call_count) = max_call_count {
                    let progress = (current_call_count as f64 / max_call_count as f64) * 100.0;
                    // debug!("{} / {} = {}", current_call_count, max_call_count, progress);
                    if 100.0 <= progress {
                        info!(
                            "{}: {}: {}: {} / {} => {}%",
                            self.id,
                            stage,
                            State::Fulfilled,
                            current_call_count,
                            max_call_count,
                            progress
                        );
                    } else {
                        info!(
                            "{}: {}: {}: {} / {} => {}%",
                            self.id, stage, state, current_call_count, max_call_count, progress
                        );
                    }
                } else {
                    info!("{}: {}: {}", self.id, stage, state);
                }
                Ok(r)
            }
            Err(err) => {
                error!("{}: {}: Error: {:#?}", self.id, stage, err);
                Err(err)
            }
        }
    }
}

pub fn update<ID, T, F>(stage_updater: &StageUpdater<ID>, stage: Stage, f: F) -> anyhow::Result<T>
where
    ID: Display,
    F: Fn() -> StageR<T>,
{
    stage_updater.update(stage, f)
}

pub type MaxCallCount = usize;

pub struct StageR<T>(pub State, pub Option<MaxCallCount>, pub anyhow::Result<T>);

pub enum State {
    Ready,
    Pending,
    Fulfilled,
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

impl PartialEq for Stage {
    fn eq(&self, other: &Stage) -> bool {
        self.as_u8() == other.as_u8()
    }
}
impl Eq for Stage {}

impl Display for Stage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

impl From<u8> for Stage {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::ParseBook,
            1 => Self::ParseImages,
            2 => Self::AddThumbnail,
            3 => Self::AddImages,
            4 => Self::AddImageList,
            5 => Self::AddBook,
            _ => panic!("Can't Stage from {}", x),
        }
    }
}

impl Stage {
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::ParseBook => 0,
            Self::ParseImages => 1,
            Self::AddThumbnail => 2,
            Self::AddImages => 3,
            Self::AddImageList => 4,
            Self::AddBook => 5,
        }
    }
}
