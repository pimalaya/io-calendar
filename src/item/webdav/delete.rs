//! WebDAV item delete coroutine wrapping
//! [`io_webdav::rfc4791::item::delete::DeleteItem`].
//!
//! # Example
//!
//! ```rust,ignore
//! // Driven through the shared-API method on the WebDAV client.
//! client.delete_item("personal", "event-1")?;
//! ```

use io_webdav::{
    coroutine::*,
    rfc4791::item::delete::DeleteItem,
    rfc4918::{WebdavAuth, send::SendError},
};
use log::trace;
use url::Url;

/// I/O-free coroutine deleting a single WebDAV item.
pub struct WebdavCalendarItemDelete {
    inner: DeleteItem,
}

impl WebdavCalendarItemDelete {
    /// Builds the coroutine deleting item `item_id` from the collection
    /// at `calendar_path`.
    pub fn new(
        base_url: &Url,
        auth: &WebdavAuth,
        user_agent: &str,
        calendar_path: &str,
        item_id: &str,
    ) -> Self {
        trace!("prepare webdav item delete");
        Self {
            inner: DeleteItem::new(base_url, auth, user_agent, calendar_path, item_id, None),
        }
    }
}

impl WebdavCoroutine for WebdavCalendarItemDelete {
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
