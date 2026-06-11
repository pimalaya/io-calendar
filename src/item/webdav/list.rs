//! WebDAV item list coroutine wrapping
//! [`io_webdav::rfc4791::item::list::ListItems`].
//!
//! Lists every item kind (the `comp_filter` is empty); per-kind
//! filtering belongs to protocol-specific commands, not the shared API.
//!
//! # Example
//!
//! ```rust,ignore
//! // Driven through the shared-API method on the WebDAV client.
//! let items = client.list_items("personal", None, None)?;
//! ```

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use io_webdav::{
    coroutine::*,
    rfc4791::item::list::ListItems,
    rfc4918::{WebdavAuth, send::SendError},
};
use log::trace;
use url::Url;

use crate::{
    item::CalendarItem,
    webdav::convert::{item_from_entry, paginate},
};

/// I/O-free coroutine listing every item inside a WebDAV calendar
/// collection.
///
/// On completion maps each wire entry to a [`CalendarItem`], sorts by
/// id, then applies 1-indexed pagination.
pub struct WebdavCalendarItemList {
    calendar_id: String,
    page: Option<u32>,
    page_size: Option<u32>,
    inner: ListItems,
}

impl WebdavCalendarItemList {
    /// Builds the coroutine listing items in the collection at
    /// `calendar_path` (the calendar `calendar_id`), applying 1-indexed
    /// pagination on completion.
    pub fn new(
        base_url: &Url,
        auth: &WebdavAuth,
        user_agent: &str,
        calendar_path: &str,
        calendar_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Self {
        trace!("prepare webdav item list");
        Self {
            calendar_id: calendar_id.to_string(),
            page,
            page_size,
            inner: ListItems::new(base_url, auth, user_agent, calendar_path, ""),
        }
    }
}

impl WebdavCoroutine for WebdavCalendarItemList {
    type Yield = WebdavYield;
    type Return = Result<Vec<CalendarItem>, SendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> WebdavCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            WebdavCoroutineState::Yielded(y) => WebdavCoroutineState::Yielded(y),
            WebdavCoroutineState::Complete(Ok(entries)) => {
                let mut items: Vec<CalendarItem> = entries
                    .into_iter()
                    .map(|entry| item_from_entry(entry, &self.calendar_id))
                    .collect();
                items.sort_by(|a, b| a.id.cmp(&b.id));
                let items = paginate(items, self.page, self.page_size);
                WebdavCoroutineState::Complete(Ok(items))
            }
            WebdavCoroutineState::Complete(Err(err)) => WebdavCoroutineState::Complete(Err(err)),
        }
    }
}
