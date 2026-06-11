//! WebDAV calendar update coroutine wrapping
//! [`io_webdav::rfc4791::calendar::update::UpdateCalendar`].
//!
//! The diff merge against the current calendar happens in the client
//! method (which lists calendars first); this coroutine only writes the
//! already-merged [`WireCalendar`].
//!
//! # Example
//!
//! ```rust,ignore
//! use io_calendar::calendar::CalendarDiff;
//!
//! // Driven through the shared-API method on the WebDAV client.
//! let patch = CalendarDiff { name: Some("Personal".into()), ..Default::default() };
//! client.update_calendar("personal", patch)?;
//! ```

use io_webdav::{
    coroutine::*,
    rfc4791::calendar::{Calendar as WireCalendar, update::UpdateCalendar},
    rfc4918::{WebdavAuth, send::SendError},
};
use log::trace;
use url::Url;

/// I/O-free coroutine updating a WebDAV calendar collection's
/// properties from an already-merged [`WireCalendar`].
pub struct WebdavCalendarUpdate {
    inner: UpdateCalendar,
}

impl WebdavCalendarUpdate {
    /// Builds the coroutine applying the already-merged `calendar` under
    /// `home_set_path`.
    pub fn new(
        base_url: &Url,
        auth: &WebdavAuth,
        user_agent: &str,
        home_set_path: &str,
        calendar: &WireCalendar,
    ) -> Self {
        trace!("prepare webdav calendar update");
        Self {
            inner: UpdateCalendar::new(base_url, auth, user_agent, home_set_path, calendar),
        }
    }
}

impl WebdavCoroutine for WebdavCalendarUpdate {
    type Yield = WebdavYield;
    type Return = Result<(), SendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> WebdavCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            WebdavCoroutineState::Yielded(y) => WebdavCoroutineState::Yielded(y),
            WebdavCoroutineState::Complete(r) => WebdavCoroutineState::Complete(r),
        }
    }
}
