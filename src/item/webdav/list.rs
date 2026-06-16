//! WebDAV item list coroutine wrapping
//! [`io_webdav::rfc4791::item::list::ListItems`].
//!
//! Lists every item kind (the `comp_filter` is empty) unless a
//! [`TimeRange`] is given, in which case the query is constrained to
//! VEVENT components overlapping the range (the only kind a time-range
//! filter applies to in the shared API).
//!
//! # Example
//!
//! ```rust,ignore
//! // Driven through the shared-API method on the WebDAV client.
//! let items = client.list_items("personal", None, None, None)?;
//! ```

use alloc::{
    format,
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
    item::{CalendarItem, TimeRange},
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
    ///
    /// When `time_range` is set, the server query is constrained to
    /// VEVENT components overlapping the range.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base_url: &Url,
        auth: &WebdavAuth,
        user_agent: &str,
        calendar_path: &str,
        calendar_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        time_range: Option<&TimeRange>,
    ) -> Self {
        trace!("prepare webdav item list");

        let comp_filter = match time_range {
            None => String::new(),
            Some(range) => {
                let mut attrs = String::new();
                if let Some(start) = range.start() {
                    attrs.push_str(&format!(" start=\"{start}\""));
                }
                if let Some(end) = range.end() {
                    attrs.push_str(&format!(" end=\"{end}\""));
                }
                format!("<C:comp-filter name=\"VEVENT\"><C:time-range{attrs} /></C:comp-filter>")
            }
        };

        Self {
            calendar_id: calendar_id.to_string(),
            page,
            page_size,
            inner: ListItems::new(base_url, auth, user_agent, calendar_path, &comp_filter),
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
