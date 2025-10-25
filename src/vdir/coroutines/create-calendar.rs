use std::path::Path;

use io_fs::io::FsIo;
use io_vdir::{
    collection::Collection,
    coroutines::create_collection::{
        CreateCollection, CreateCollectionError, CreateCollectionResult,
    },
};
use thiserror::Error;

use crate::calendar::Calendar;

#[derive(Clone, Debug, Error)]
pub enum CreateCalendarError {
    #[error("Create calendar error")]
    CreateItem(#[from] CreateCollectionError),
}

#[derive(Clone, Debug)]
pub enum CreateCalendarResult {
    Ok,
    Err(CreateCalendarError),
    Io(FsIo),
}

#[derive(Debug)]
pub struct CreateCalendar(CreateCollection);

impl CreateCalendar {
    pub fn new(root: impl AsRef<Path>, calendar: Calendar) -> Self {
        Self(CreateCollection::new(Collection {
            path: root.as_ref().join(calendar.id),
            display_name: calendar.display_name,
            description: calendar.description,
            color: calendar.color,
        }))
    }

    pub fn resume(&mut self, input: Option<FsIo>) -> CreateCalendarResult {
        match self.0.resume(input) {
            CreateCollectionResult::Ok => CreateCalendarResult::Ok,
            CreateCollectionResult::Err(err) => CreateCalendarResult::Err(err.into()),
            CreateCollectionResult::Io(io) => CreateCalendarResult::Io(io),
        }
    }
}
