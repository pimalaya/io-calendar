//! WebDAV calendar create coroutine wrapping
//! [`io_webdav::rfc4791::calendar::create::CreateCalendar`].
//!
//! # Example
//!
//! ```rust,ignore
//! // Driven through the shared-API method on the WebDAV client.
//! client.create_calendar("personal", "Personal", None, None)?;
//! ```

use alloc::string::String;

use io_webdav::{
    coroutine::*,
    rfc4791::calendar::{Calendar as WireCalendar, create::CreateCalendar},
    rfc4918::{WebdavAuth, send::SendError},
};
use log::trace;
use url::Url;

/// I/O-free coroutine creating a WebDAV calendar collection.
///
/// On completion returns the new calendar id (its URL segment).
pub struct WebdavCalendarCreate {
    id: String,
    inner: CreateCalendar,
}

impl WebdavCalendarCreate {
    /// Builds the coroutine creating `calendar` under `home_set_path`.
    pub fn new(
        base_url: &Url,
        auth: &WebdavAuth,
        user_agent: &str,
        home_set_path: &str,
        calendar: &WireCalendar,
    ) -> Self {
        trace!("prepare webdav calendar create");
        Self {
            id: calendar.id.clone(),
            inner: CreateCalendar::new(base_url, auth, user_agent, home_set_path, calendar),
        }
    }
}

impl WebdavCoroutine for WebdavCalendarCreate {
    type Yield = WebdavYield;
    type Return = Result<String, SendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> WebdavCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            WebdavCoroutineState::Yielded(y) => WebdavCoroutineState::Yielded(y),
            WebdavCoroutineState::Complete(Ok(())) => {
                WebdavCoroutineState::Complete(Ok(self.id.clone()))
            }
            WebdavCoroutineState::Complete(Err(err)) => WebdavCoroutineState::Complete(Err(err)),
        }
    }
}
