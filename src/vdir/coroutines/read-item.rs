use std::path::{Path, PathBuf};

use io_fs::io::FsIo;
use io_vdir::{
    constants::ICS,
    coroutines::read_item::{
        ReadItem as ReadVdirItem, ReadItemError as ReadVdirItemError,
        ReadItemResult as ReadVdirItemResult,
    },
    item::ItemKind,
};
use thiserror::Error;

use crate::item::CalendarItem;

#[derive(Clone, Debug, Error)]
pub enum ReadItemError {
    #[error("Read calendar vdir item error")]
    ReadVdirItem(#[from] ReadVdirItemError),
    #[error("Invalid calendar item path {0}")]
    InvalidCalendarItemPath(PathBuf),
    #[error("Invalid calendar id at {0}")]
    InvalidCalendarId(PathBuf),
    #[error("Invalid calendar item id at {0}")]
    InvalidCalendarItemId(PathBuf),
    #[error("Invalid calendar item at {0}")]
    InvalidCalendarItem(PathBuf),
}

#[derive(Clone, Debug)]
pub enum ReadCalendarItemResult {
    Ok(CalendarItem),
    Err(ReadItemError),
    Io(FsIo),
}

#[derive(Debug)]
pub struct ReadCalendarItem(ReadVdirItem);

impl ReadCalendarItem {
    pub fn new(root: impl AsRef<Path>, calendar_id: impl AsRef<str>, id: impl AsRef<str>) -> Self {
        let path = root
            .as_ref()
            .join(calendar_id.as_ref())
            .join(id.as_ref())
            .with_extension(ICS);

        Self(ReadVdirItem::new(path))
    }

    pub fn resume(&mut self, input: Option<FsIo>) -> ReadCalendarItemResult {
        let item = loop {
            match self.0.resume(input) {
                ReadVdirItemResult::Ok(item) => break item,
                ReadVdirItemResult::Err(err) => return ReadCalendarItemResult::Err(err.into()),
                ReadVdirItemResult::Io(io) => return ReadCalendarItemResult::Io(io),
            }
        };

        let p = &item.path;

        let Some(parent) = p.parent() else {
            return ReadCalendarItemResult::Err(ReadItemError::InvalidCalendarItemPath(
                p.to_owned(),
            ));
        };

        let Some(calendar_id) = parent.file_stem() else {
            return ReadCalendarItemResult::Err(ReadItemError::InvalidCalendarId(p.to_owned()));
        };

        let Some(id) = p.file_stem() else {
            return ReadCalendarItemResult::Err(ReadItemError::InvalidCalendarItemId(p.to_owned()));
        };

        let ItemKind::Ical(ical) = item.kind else {
            return ReadCalendarItemResult::Err(ReadItemError::InvalidCalendarItem(p.to_owned()));
        };

        let item = CalendarItem {
            id: id.to_string_lossy().to_string(),
            calendar_id: calendar_id.to_string_lossy().to_string(),
            ical,
        };

        ReadCalendarItemResult::Ok(item)
    }
}
