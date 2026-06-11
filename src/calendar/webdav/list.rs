//! WebDAV calendar list coroutine wrapping
//! [`io_webdav::rfc4791::calendar::list::ListCalendars`].
//!
//! # Example
//!
//! ```rust,ignore
//! // Driven through the shared-API method on the WebDAV client.
//! let calendars = client.list_calendars()?;
//! ```

use alloc::vec::Vec;

use io_webdav::{
    coroutine::*,
    rfc4791::calendar::list::ListCalendars,
    rfc4918::{WebdavAuth, send::SendError},
};
use log::trace;
use url::Url;

use crate::calendar::Calendar;

/// I/O-free coroutine listing every WebDAV calendar under the home-set.
///
/// On completion maps each wire calendar to a [`Calendar`] and
/// sorts the result by name.
pub struct WebdavCalendarList {
    inner: ListCalendars,
}

impl WebdavCalendarList {
    /// Builds the coroutine listing calendars under `home_set_path`.
    pub fn new(base_url: &Url, auth: &WebdavAuth, user_agent: &str, home_set_path: &str) -> Self {
        trace!("prepare webdav calendar list");
        Self {
            inner: ListCalendars::new(base_url, auth, user_agent, home_set_path),
        }
    }
}

impl WebdavCoroutine for WebdavCalendarList {
    type Yield = WebdavYield;
    type Return = Result<Vec<Calendar>, SendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> WebdavCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            WebdavCoroutineState::Yielded(y) => WebdavCoroutineState::Yielded(y),
            WebdavCoroutineState::Complete(Ok(wires)) => {
                let mut calendars: Vec<Calendar> = wires.into_iter().map(Calendar::from).collect();
                calendars.sort_by(|a, b| a.name.cmp(&b.name));
                WebdavCoroutineState::Complete(Ok(calendars))
            }
            WebdavCoroutineState::Complete(Err(err)) => WebdavCoroutineState::Complete(Err(err)),
        }
    }
}
