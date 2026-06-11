//! WebDAV item update coroutine wrapping
//! [`io_webdav::rfc4791::item::update::UpdateItem`].
//!
//! # Example
//!
//! ```rust,ignore
//! // Driven through the shared-API method on the WebDAV client.
//! client.update_item("personal", "event-1", ical_bytes, None)?;
//! ```

use alloc::vec::Vec;

use io_webdav::{
    coroutine::*,
    rfc4791::item::update::UpdateItem,
    rfc4918::{WebdavAuth, send::SendError},
};
use log::trace;
use url::Url;

/// I/O-free coroutine overwriting an existing WebDAV item.
pub struct WebdavCalendarItemUpdate {
    inner: UpdateItem,
}

impl WebdavCalendarItemUpdate {
    /// Builds the coroutine overwriting item `item_id` in the collection
    /// at `calendar_path` with `contents`, gating the write on
    /// `if_match` when present.
    pub fn new(
        base_url: &Url,
        auth: &WebdavAuth,
        user_agent: &str,
        calendar_path: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Self {
        trace!("prepare webdav item update");
        Self {
            inner: UpdateItem::new(
                base_url,
                auth,
                user_agent,
                calendar_path,
                item_id,
                contents,
                if_match,
            ),
        }
    }
}

impl WebdavCoroutine for WebdavCalendarItemUpdate {
    type Yield = WebdavYield;
    type Return = Result<(), SendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> WebdavCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            WebdavCoroutineState::Yielded(y) => WebdavCoroutineState::Yielded(y),
            WebdavCoroutineState::Complete(Ok(_)) => WebdavCoroutineState::Complete(Ok(())),
            WebdavCoroutineState::Complete(Err(err)) => WebdavCoroutineState::Complete(Err(err)),
        }
    }
}
