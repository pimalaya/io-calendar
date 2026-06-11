//! WebDAV item create coroutine wrapping
//! [`io_webdav::rfc4791::item::create::CreateItem`].
//!
//! # Example
//!
//! ```rust,ignore
//! // Driven through the shared-API method on the WebDAV client.
//! let id = client.create_item("personal", ical_bytes)?;
//! ```

use alloc::{string::String, vec::Vec};

use io_webdav::{
    coroutine::*,
    rfc4791::item::create::CreateItem,
    rfc4918::{WebdavAuth, send::SendError},
};
use log::trace;
use url::Url;

/// I/O-free coroutine creating a WebDAV item under an `id` minted by the
/// caller.
///
/// On completion returns the id the server confirmed.
pub struct WebdavCalendarItemCreate {
    inner: CreateItem,
}

impl WebdavCalendarItemCreate {
    /// Builds the coroutine storing `contents` as item `id` inside the
    /// collection at `calendar_path`. The id is synthesized by the
    /// caller: the shared API does not parse the iCalendar UID.
    pub fn new(
        base_url: &Url,
        auth: &WebdavAuth,
        user_agent: &str,
        calendar_path: &str,
        id: &str,
        contents: Vec<u8>,
    ) -> Self {
        trace!("prepare webdav item create");
        Self {
            inner: CreateItem::new(base_url, auth, user_agent, calendar_path, id, contents),
        }
    }
}

impl WebdavCoroutine for WebdavCalendarItemCreate {
    type Yield = WebdavYield;
    type Return = Result<String, SendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> WebdavCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            WebdavCoroutineState::Yielded(y) => WebdavCoroutineState::Yielded(y),
            WebdavCoroutineState::Complete(Ok(ok)) => WebdavCoroutineState::Complete(Ok(ok.id)),
            WebdavCoroutineState::Complete(Err(err)) => WebdavCoroutineState::Complete(Err(err)),
        }
    }
}
