//! Vdir item list coroutine wrapping
//! [`io_vdir::item::list::VdirItemList`].
//!
//! Filters to iCalendar items, sorts by id, then applies 1-indexed
//! pagination.
//!
//! # Example
//!
//! ```rust,ignore
//! use io_calendar::{item::vdir::list::VdirCalendarItemList, vdir::client::VdirClient};
//! use io_vdir::path::VdirPath;
//!
//! let client = VdirClient::new("/path/to/vdir");
//! let path = client.inner.root().join("personal");
//! let items = client.run(VdirCalendarItemList::new(path, "personal", None, None))?;
//! ```

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use io_vdir::{
    coroutine::*,
    item::list::{VdirItemList, VdirItemListError, VdirItemListOptions},
    path::VdirPath,
};
use log::trace;
use thiserror::Error;

use crate::{
    item::CalendarItem,
    vdir::convert::{is_calendar_item, item_from, paginate},
};

/// Errors produced by [`VdirCalendarItemList`].
#[derive(Debug, Error)]
pub enum VdirCalendarItemListError {
    #[error(transparent)]
    List(#[from] VdirItemListError),
}

/// I/O-free coroutine listing every iCalendar item in a Vdir calendar.
///
/// On completion keeps only iCalendar items, maps each to a
/// [`CalendarItem`], sorts by id, then paginates.
pub struct VdirCalendarItemList {
    calendar_id: String,
    page: Option<u32>,
    page_size: Option<u32>,
    inner: VdirItemList,
}

impl VdirCalendarItemList {
    /// Builds the coroutine listing items of calendar `calendar_id`
    /// located at `path`, applying 1-indexed pagination on completion.
    pub fn new(
        path: impl Into<VdirPath>,
        calendar_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Self {
        trace!("prepare vdir item list");
        Self {
            calendar_id: calendar_id.to_string(),
            page,
            page_size,
            inner: VdirItemList::new(path, VdirItemListOptions::default()),
        }
    }
}

impl VdirCoroutine for VdirCalendarItemList {
    type Yield = VdirYield;
    type Return = Result<Vec<CalendarItem>, VdirCalendarItemListError>;

    fn resume(&mut self, arg: Option<VdirReply>) -> VdirCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            VdirCoroutineState::Yielded(y) => VdirCoroutineState::Yielded(y),
            VdirCoroutineState::Complete(Ok(items)) => {
                let mut items: Vec<CalendarItem> = items
                    .into_iter()
                    .filter(|item| is_calendar_item(item.kind))
                    .filter_map(|item| item_from(item, &self.calendar_id))
                    .collect();
                items.sort_by(|a, b| a.id.cmp(&b.id));
                let items = paginate(items, self.page, self.page_size);
                VdirCoroutineState::Complete(Ok(items))
            }
            VdirCoroutineState::Complete(Err(err)) => VdirCoroutineState::Complete(Err(err.into())),
        }
    }
}
