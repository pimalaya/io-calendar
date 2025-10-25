use std::{collections::HashSet, path::Path};

use io_fs::io::FsIo;
use io_vdir::{
    coroutines::list_items::{
        ListItems as ListVdirItems, ListItemsError as ListVdirItemsError,
        ListItemsResult as ListVdirItemsResult,
    },
    item::ItemKind,
};
use thiserror::Error;

use crate::item::CalendarItem;

#[derive(Clone, Debug, Error)]
pub enum ListCalendarItemsError {
    #[error("List cards error")]
    ListVdirItems(#[from] ListVdirItemsError),
}

#[derive(Clone, Debug)]
pub enum ListCalendarItemsResult {
    Ok(HashSet<CalendarItem>),
    Err(ListCalendarItemsError),
    Io(FsIo),
}

#[derive(Debug)]
pub struct ListCalendarItems(ListVdirItems);

impl ListCalendarItems {
    pub fn new(root: impl AsRef<Path>, calendar_id: impl AsRef<str>) -> Self {
        Self(ListVdirItems::new(root.as_ref().join(calendar_id.as_ref())))
    }

    pub fn resume(&mut self, input: Option<FsIo>) -> ListCalendarItemsResult {
        let items = loop {
            match self.0.resume(input) {
                ListVdirItemsResult::Ok(items) => break items,
                ListVdirItemsResult::Err(err) => return ListCalendarItemsResult::Err(err.into()),
                ListVdirItemsResult::Io(io) => return ListCalendarItemsResult::Io(io),
            }
        };

        let mut cards = HashSet::new();

        for item in items {
            let Some(parent) = item.path.parent() else {
                continue;
            };

            let Some(calendar_id) = parent.file_stem() else {
                continue;
            };

            let Some(id) = item.path.file_stem() else {
                continue;
            };

            let ItemKind::Ical(ical) = item.kind else {
                continue;
            };

            cards.insert(CalendarItem {
                id: id.to_string_lossy().to_string(),
                calendar_id: calendar_id.to_string_lossy().to_string(),
                ical,
            });
        }

        ListCalendarItemsResult::Ok(cards)
    }
}
