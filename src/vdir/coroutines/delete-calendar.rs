use std::path::Path;

use io_fs::io::FsIo;
use io_vdir::coroutines::delete_collection::{
    DeleteCollection, DeleteCollectionError, DeleteCollectionResult,
};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum DeleteCalendarError {
    #[error("Delete calendar error")]
    DeleteItem(#[from] DeleteCollectionError),
}

#[derive(Clone, Debug)]
pub enum DeleteCalendarResult {
    Ok,
    Err(DeleteCalendarError),
    Io(FsIo),
}

#[derive(Debug)]
pub struct DeleteCalendar(DeleteCollection);

impl DeleteCalendar {
    pub fn new(root: impl AsRef<Path>, id: impl AsRef<str>) -> Self {
        Self(DeleteCollection::new(root.as_ref().join(id.as_ref())))
    }

    pub fn resume(&mut self, input: Option<FsIo>) -> DeleteCalendarResult {
        match self.0.resume(input) {
            DeleteCollectionResult::Ok => DeleteCalendarResult::Ok,
            DeleteCollectionResult::Err(err) => DeleteCalendarResult::Err(err.into()),
            DeleteCollectionResult::Io(io) => DeleteCalendarResult::Io(io),
        }
    }
}
