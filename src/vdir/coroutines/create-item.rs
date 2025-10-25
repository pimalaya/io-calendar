use std::path::Path;

use io_fs::io::FsIo;
use io_vdir::{
    constants::ICS,
    coroutines::create_item::{
        CreateItem as CreateVdirItem, CreateItemError as CreateVdirItemError,
        CreateItemResult as CreateVdirItemResult,
    },
    item::{Item as VdirItem, ItemKind},
};
use thiserror::Error;

use crate::item::CalendarItem;

#[derive(Clone, Debug, Error)]
pub enum CreateCalendarItemError {
    #[error("Create calendar item error")]
    CreateVdirItem(#[from] CreateVdirItemError),
}

#[derive(Clone, Debug)]
pub enum CreateCalendarItemResult {
    Ok,
    Err(CreateCalendarItemError),
    Io(FsIo),
}

#[derive(Debug)]
pub struct CreateCalendarItem(CreateVdirItem);

impl CreateCalendarItem {
    pub fn new(root: impl AsRef<Path>, item: CalendarItem) -> Self {
        let kind = ItemKind::Ical(item.ical);
        let path = root
            .as_ref()
            .join(item.calendar_id)
            .join(item.id)
            .with_extension(ICS);

        Self(CreateVdirItem::new(VdirItem { path, kind }))
    }

    pub fn resume(&mut self, input: Option<FsIo>) -> CreateCalendarItemResult {
        match self.0.resume(input) {
            CreateVdirItemResult::Ok => CreateCalendarItemResult::Ok,
            CreateVdirItemResult::Err(err) => CreateCalendarItemResult::Err(err.into()),
            CreateVdirItemResult::Io(io) => CreateCalendarItemResult::Io(io),
        }
    }
}
