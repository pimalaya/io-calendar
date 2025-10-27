use std::path::Path;

use io_fs::io::FsIo;
use io_vdir::{
    constants::ICS,
    coroutines::delete_item::{
        DeleteItem as DeleteVdirItem, DeleteItemError as DeleteVdirItemError,
        DeleteItemResult as DeleteVdirItemResult,
    },
};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum DeleteCalendarItemError {
    #[error("Delete calendar item error")]
    DeleteVdirItem(#[from] DeleteVdirItemError),
}

#[derive(Clone, Debug)]
pub enum DeleteCalendarItemResult {
    Ok,
    Err(DeleteCalendarItemError),
    Io(FsIo),
}

#[derive(Debug)]
pub struct DeleteCalendarItem(DeleteVdirItem);

impl DeleteCalendarItem {
    pub fn new(root: impl AsRef<Path>, calendar_id: impl AsRef<str>, id: impl AsRef<str>) -> Self {
        let path = root
            .as_ref()
            .join(calendar_id.as_ref())
            .join(id.as_ref())
            .with_extension(ICS);

        Self(DeleteVdirItem::new(path))
    }

    pub fn resume(&mut self, input: Option<FsIo>) -> DeleteCalendarItemResult {
        match self.0.resume(input) {
            DeleteVdirItemResult::Ok => DeleteCalendarItemResult::Ok,
            DeleteVdirItemResult::Err(err) => DeleteCalendarItemResult::Err(err.into()),
            DeleteVdirItemResult::Io(io) => DeleteCalendarItemResult::Io(io),
        }
    }
}
