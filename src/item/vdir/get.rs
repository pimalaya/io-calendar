//! Vdir item get coroutine wrapping
//! [`io_vdir::item::get::VdirItemGet`].
//!
//! # Example
//!
//! ```rust,ignore
//! use io_calendar::{item::vdir::get::VdirCalendarItemGet, vdir::client::VdirClient};
//!
//! let client = VdirClient::new("/path/to/vdir");
//! let path = client.inner.root().join("personal");
//! let item = client.run(VdirCalendarItemGet::new(path, "personal", "event-1"))?;
//! ```

use alloc::string::{String, ToString};

use io_vdir::{
    coroutine::*,
    item::get::{VdirItemGet, VdirItemGetError, VdirItemGetOptions},
    path::VdirPath,
};
use log::trace;
use thiserror::Error;

use crate::{
    item::CalendarItem,
    vdir::convert::{is_calendar_item, item_from},
};

/// Errors produced by [`VdirCalendarItemGet`].
#[derive(Debug, Error)]
pub enum VdirCalendarItemGetError {
    #[error(transparent)]
    Get(#[from] VdirItemGetError),
    #[error("Invalid item id `{0}`")]
    InvalidId(String),
}

/// I/O-free coroutine fetching a Vdir item by its id.
///
/// On completion the located item is converted to a [`CalendarItem`]; a
/// non-iCalendar item or an item with no derivable id is rejected.
pub struct VdirCalendarItemGet {
    calendar_id: String,
    item_id: String,
    inner: VdirItemGet,
}

impl VdirCalendarItemGet {
    /// Builds the coroutine fetching item `item_id` from calendar
    /// `calendar_id` located at `path`.
    pub fn new(path: impl Into<VdirPath>, calendar_id: &str, item_id: &str) -> Self {
        trace!("prepare vdir item get");
        Self {
            calendar_id: calendar_id.to_string(),
            item_id: item_id.to_string(),
            inner: VdirItemGet::new(path, item_id, VdirItemGetOptions::default()),
        }
    }
}

impl VdirCoroutine for VdirCalendarItemGet {
    type Yield = VdirYield;
    type Return = Result<CalendarItem, VdirCalendarItemGetError>;

    fn resume(&mut self, arg: Option<VdirReply>) -> VdirCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            VdirCoroutineState::Yielded(y) => VdirCoroutineState::Yielded(y),
            VdirCoroutineState::Complete(Ok(item)) => {
                if !is_calendar_item(item.kind) {
                    let err = VdirCalendarItemGetError::InvalidId(self.item_id.clone());
                    return VdirCoroutineState::Complete(Err(err));
                }

                match item_from(item, &self.calendar_id) {
                    Some(item) => VdirCoroutineState::Complete(Ok(item)),
                    None => {
                        let err = VdirCalendarItemGetError::InvalidId(self.item_id.clone());
                        VdirCoroutineState::Complete(Err(err))
                    }
                }
            }
            VdirCoroutineState::Complete(Err(err)) => VdirCoroutineState::Complete(Err(err.into())),
        }
    }
}
