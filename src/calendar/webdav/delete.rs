//! WebDAV calendar delete coroutine wrapping
//! [`io_webdav::rfc4791::calendar::delete::DeleteCalendar`].
//!
//! # Example
//!
//! ```rust,ignore
//! // Driven through the shared-API method on the WebDAV client.
//! client.delete_calendar("personal")?;
//! ```

use io_webdav::{
    coroutine::*,
    rfc4791::calendar::delete::DeleteCalendar,
    rfc4918::{WebdavAuth, send::SendError},
};
use log::trace;
use url::Url;

/// I/O-free coroutine deleting a WebDAV calendar collection.
pub struct WebdavCalendarDelete {
    inner: DeleteCalendar,
}

impl WebdavCalendarDelete {
    /// Builds the coroutine deleting calendar `id` under
    /// `home_set_path`.
    pub fn new(
        base_url: &Url,
        auth: &WebdavAuth,
        user_agent: &str,
        home_set_path: &str,
        id: &str,
    ) -> Self {
        trace!("prepare webdav calendar delete");
        Self {
            inner: DeleteCalendar::new(base_url, auth, user_agent, home_set_path, id),
        }
    }
}

impl WebdavCoroutine for WebdavCalendarDelete {
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
