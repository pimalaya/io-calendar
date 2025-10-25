use std::path::Path;

use io_fs::io::FsIo;
use io_vdir::{
    constants::ICS,
    coroutines::update_item::{
        UpdateItem as UpdateVdirItem, UpdateItemError as UpdateVdirItemError,
        UpdateItemResult as UpdateVdirItemResult,
    },
    item::{Item as VdirItem, ItemKind as VdirItemKind},
};
use thiserror::Error;

use crate::item::CalendarItem;

#[derive(Clone, Debug, Error)]
pub enum UpdateCalendarItemError {
    #[error("Update calendar vdir item error")]
    UpdateVdirItem(#[from] UpdateVdirItemError),
}

#[derive(Clone, Debug)]
pub enum UpdateCalendarItemResult {
    Ok,
    Err(UpdateCalendarItemError),
    Io(FsIo),
}

#[derive(Debug)]
pub struct UpdateCalendarItem(UpdateVdirItem);

impl UpdateCalendarItem {
    pub fn new(root: impl AsRef<Path>, item: CalendarItem) -> Self {
        let kind = VdirItemKind::Ical(item.ical);
        let path = root
            .as_ref()
            .join(item.calendar_id)
            .join(item.id)
            .with_extension(ICS);

        Self(UpdateVdirItem::new(VdirItem { path, kind }))
    }

    pub fn resume(&mut self, input: Option<FsIo>) -> UpdateCalendarItemResult {
        match self.0.resume(input) {
            UpdateVdirItemResult::Ok => UpdateCalendarItemResult::Ok,
            UpdateVdirItemResult::Err(err) => UpdateCalendarItemResult::Err(err.into()),
            UpdateVdirItemResult::Io(io) => UpdateCalendarItemResult::Io(io),
        }
    }
}
