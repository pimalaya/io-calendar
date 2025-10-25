use std::path::Path;

use io_fs::io::FsIo;
use io_vdir::{
    collection::Collection,
    coroutines::update_collection::{
        UpdateCollection, UpdateCollectionError, UpdateCollectionResult,
    },
};
use thiserror::Error;

use crate::calendar::Calendar;

#[derive(Clone, Debug, Error)]
pub enum UpdateCalendarError {
    #[error("Update calendar error")]
    UpdateCollection(#[from] UpdateCollectionError),
}

#[derive(Clone, Debug)]
pub enum UpdateCalendarResult {
    Ok,
    Err(UpdateCalendarError),
    Io(FsIo),
}

#[derive(Debug)]
pub struct UpdateCalendar(UpdateCollection);

impl UpdateCalendar {
    pub fn new(root: impl AsRef<Path>, calendar: Calendar) -> Self {
        Self(UpdateCollection::new(Collection {
            path: root.as_ref().join(calendar.id),
            display_name: calendar.display_name,
            description: calendar.description,
            color: calendar.color,
        }))
    }

    pub fn resume(&mut self, input: Option<FsIo>) -> UpdateCalendarResult {
        match self.0.resume(input) {
            UpdateCollectionResult::Ok => UpdateCalendarResult::Ok,
            UpdateCollectionResult::Err(err) => UpdateCalendarResult::Err(err.into()),
            UpdateCollectionResult::Io(io) => UpdateCalendarResult::Io(io),
        }
    }
}
