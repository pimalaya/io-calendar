//! WebDAV item get coroutine wrapping
//! [`io_webdav::rfc4791::item::read::ReadItem`].
//!
//! # Example
//!
//! ```rust,ignore
//! // Driven through the shared-API method on the WebDAV client.
//! let item = client.get_item("personal", "event-1")?;
//! ```

use alloc::string::{String, ToString};

use io_webdav::{
    coroutine::*,
    rfc4791::item::read::ReadItem,
    rfc4918::{WebdavAuth, send::SendError},
};
use log::trace;
use url::Url;

use crate::{item::CalendarItem, webdav::convert::item_from_body};

/// I/O-free coroutine reading a single WebDAV item by id.
///
/// On completion builds a [`CalendarItem`] from the fetched body and
/// ETag.
pub struct WebdavCalendarItemGet {
    calendar_id: String,
    item_id: String,
    inner: ReadItem,
}

impl WebdavCalendarItemGet {
    /// Builds the coroutine reading item `item_id` from the collection
    /// at `calendar_path` (the calendar `calendar_id`).
    pub fn new(
        base_url: &Url,
        auth: &WebdavAuth,
        user_agent: &str,
        calendar_path: &str,
        calendar_id: &str,
        item_id: &str,
    ) -> Self {
        trace!("prepare webdav item get");
        Self {
            calendar_id: calendar_id.to_string(),
            item_id: item_id.to_string(),
            inner: ReadItem::new(base_url, auth, user_agent, calendar_path, item_id),
        }
    }
}

impl WebdavCoroutine for WebdavCalendarItemGet {
    type Yield = WebdavYield;
    type Return = Result<CalendarItem, SendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> WebdavCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            WebdavCoroutineState::Yielded(y) => WebdavCoroutineState::Yielded(y),
            WebdavCoroutineState::Complete(Ok(body)) => {
                let item = item_from_body(body, &self.calendar_id, &self.item_id);
                WebdavCoroutineState::Complete(Ok(item))
            }
            WebdavCoroutineState::Complete(Err(err)) => WebdavCoroutineState::Complete(Err(err)),
        }
    }
}
