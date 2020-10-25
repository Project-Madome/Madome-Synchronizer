use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::sync::Mutex;
use std::time;

use log::{error, info};
use rdkafka::producer::BaseProducer;
use serde::Serialize;

use super::kafka::producer;

/* #[derive(Serialize)]
pub struct TaskAddPayload<'a, ID>
where
    ID: Display + Serialize,
{
    id: &'a ID,
}

impl<'a, ID> TaskAddPayload<'a, ID>
where
    ID: Display + Serialize,
{
    pub fn new(id: &'a ID) -> Self {
        Self { id }
    }
} */

#[derive(Serialize)]
pub struct TaskUpdatePayload<'a, ID>
where
    ID: Display + Serialize,
{
    id: &'a ID,
    stage: &'a Stage,
    state: &'a State,
    progress: Option<f64>,
}

impl<'a, ID> TaskUpdatePayload<'a, ID>
where
    ID: Display + Serialize,
{
    pub fn new(id: &'a ID, stage: &'a Stage, state: &'a State, progress: Option<f64>) -> Self {
        Self {
            id,
            stage,
            state,
            progress,
        }
    }
}

#[derive(Serialize)]
pub struct TaskErrorPayload<'a, ID, E>
where
    ID: Display + Serialize,
    E: Serialize,
{
    id: &'a ID,
    stage: &'a Stage,
    error: &'a E,
}

impl<'a, ID, E> TaskErrorPayload<'a, ID, E>
where
    ID: Display + Serialize,
    E: Serialize,
{
    pub fn new(id: &'a ID, stage: &'a Stage, error: &'a E) -> Self {
        Self { id, stage, error }
    }
}

pub struct StageUpdater<ID>
where
    ID: Display + Serialize,
{
    id: ID,
    inner: Mutex<HashMap<u8, usize>>,
    producer: BaseProducer,
}

impl<ID> StageUpdater<ID>
where
    ID: Display + Serialize,
{
    pub fn new(id: ID) -> Self {
        let producer = match producer::create(vec!["192.168.0.21:9092"]) {
            Ok(producer) => producer,
            Err(err) => panic!("{:#?}", err),
        };

        Self {
            id,
            inner: Mutex::new(HashMap::new()),
            producer,
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
                let payload = TaskUpdatePayload::new(&self.id, &stage, &State::Ready, None);
                producer::send(
                    &self.producer,
                    "task-update",
                    &(),
                    &serde_json::to_string(&payload)?,
                )?;
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

                    let state = if 100.0 <= progress {
                        State::Fulfilled
                    } else {
                        state
                    };

                    let payload = TaskUpdatePayload::new(&self.id, &stage, &state, Some(progress));
                    producer::send(
                        &self.producer,
                        "task-update",
                        &(),
                        &serde_json::to_string(&payload)?,
                    )?;
                    info!(
                        "{}: {}: {}: {} / {} => {}%",
                        self.id, stage, state, current_call_count, max_call_count, progress
                    );
                } else {
                    let payload = TaskUpdatePayload::new(&self.id, &stage, &state, None);
                    producer::send(
                        &self.producer,
                        "task-update",
                        &(),
                        &serde_json::to_string(&payload)?,
                    )?;
                    info!("{}: {}: {}", self.id, stage, state);
                    self.producer.flush(time::Duration::from_secs(2));
                }
                Ok(r)
            }
            Err(err) => {
                let err_msg = format!("{:#?}", err);
                let payload = TaskErrorPayload::new(&self.id, &stage, &err_msg);
                producer::send(
                    &self.producer,
                    "task-error",
                    &(),
                    &serde_json::to_string(&payload)?,
                )?;
                error!("{}: {}: Error: {:#?}", self.id, stage, err);
                self.producer.flush(time::Duration::from_secs(2));
                Err(err)
            }
        }
    }
}

pub fn update<ID, T, F>(stage_updater: &StageUpdater<ID>, stage: Stage, f: F) -> anyhow::Result<T>
where
    ID: Display + Serialize,
    F: Fn() -> StageR<T>,
{
    stage_updater.update(stage, f)
}

pub type MaxCallCount = usize;

pub struct StageR<T>(pub State, pub Option<MaxCallCount>, pub anyhow::Result<T>);

#[derive(Serialize)]
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

#[derive(Serialize)]
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
