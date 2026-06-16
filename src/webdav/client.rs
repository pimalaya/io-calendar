//! Std-blocking WebDAV (CalDAV) calendar client.
//!
//! Wraps an inner [`io_webdav::client::WebdavClientStd`] (the connected
//! stream plus discovery cache) and pumps io-calendar WebDAV coroutines
//! directly against the inner client's public stream, reusing its
//! discovery cache. Each shared-API method first resolves the cached
//! CalDAV home-set (running discovery on the first call), then builds
//! and runs the matching coroutine; the inner client stays reachable
//! through [`WebdavClientStd::inner`].

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use std::io::{Read, Write};

use io_webdav::{
    client::WebdavClientStd as InnerWebdavClientStd,
    coroutine::{WebdavCoroutine, WebdavCoroutineState, WebdavYield},
    rfc4791::calendar::Calendar as WireCalendar,
    rfc4918::send::SendError,
};
use thiserror::Error;

use crate::{
    calendar::{
        Calendar, CalendarDiff,
        webdav::{
            create::WebdavCalendarCreate, delete::WebdavCalendarDelete, list::WebdavCalendarList,
            update::WebdavCalendarUpdate,
        },
    },
    item::{
        CalendarItem, TimeRange,
        webdav::{
            create::WebdavCalendarItemCreate, delete::WebdavCalendarItemDelete,
            get::WebdavCalendarItemGet, list::WebdavCalendarItemList,
            update::WebdavCalendarItemUpdate,
        },
    },
    webdav::convert::{calendar_path, fresh_item_id},
};

/// Socket read-buffer size for the WebDAV run loop.
const READ_BUFFER_SIZE: usize = 16 * 1024;

/// Errors surfaced by [`WebdavClientStd`].
///
/// Flattens the inner client error (discovery plus per-request I/O) and
/// adds the domain validation failures from the client methods.
#[derive(Debug, Error)]
pub enum WebdavClientError {
    #[error(transparent)]
    Inner(#[from] io_webdav::client::WebdavClientStdError),
    #[error(transparent)]
    Send(#[from] SendError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Invalid calendar `{0}`")]
    InvalidCalendar(String),
    #[error("Invalid item id `{0}`")]
    InvalidItemId(String),
    #[error("Calendar `{0}` not found")]
    CalendarNotFound(String),
    #[error("Item body is empty")]
    EmptyItemBody,
    #[error("Failed to {0}")]
    OperationFailed(&'static str),
}

/// Std-blocking WebDAV (CalDAV) calendar client built on a connected
/// stream.
#[derive(Debug)]
pub struct WebdavClientStd {
    pub inner: InnerWebdavClientStd,
}

impl WebdavClientStd {
    /// Wraps an already-built inner client.
    pub fn new(inner: InnerWebdavClientStd) -> Self {
        Self { inner }
    }

    /// Pumps any standard-shape WebDAV coroutine (`Yield =
    /// WebdavYield`) against the inner client's connected stream until
    /// it terminates.
    ///
    /// Drives the stream directly rather than delegating to the inner
    /// client so failures route through [`WebdavClientError`]; the
    /// inner discovery cache is still resolved up front by each
    /// shared-API method.
    fn run<C, T, E>(&mut self, mut coroutine: C) -> Result<T, WebdavClientError>
    where
        C: WebdavCoroutine<Yield = WebdavYield, Return = Result<T, E>>,
        WebdavClientError: From<E>,
    {
        let mut buf = [0u8; READ_BUFFER_SIZE];
        let mut arg: Option<&[u8]> = None;

        loop {
            match coroutine.resume(arg.take()) {
                WebdavCoroutineState::Complete(Ok(out)) => return Ok(out),
                WebdavCoroutineState::Complete(Err(err)) => return Err(err.into()),
                WebdavCoroutineState::Yielded(WebdavYield::WantsRead) => {
                    let n = self.inner.stream.read(&mut buf)?;
                    arg = Some(&buf[..n]);
                }
                WebdavCoroutineState::Yielded(WebdavYield::WantsWrite(bytes)) => {
                    self.inner.stream.write_all(&bytes)?;
                }
            }
        }
    }

    /// Lists every calendar under the discovered home-set, sorted by
    /// name.
    pub fn list_calendars(&mut self) -> Result<Vec<Calendar>, WebdavClientError> {
        let home = self.inner.calendar_home_set()?;
        let home_path = home.path().to_string();

        let coroutine = WebdavCalendarList::new(
            &self.inner.base_url,
            self.inner.auth(),
            &self.inner.user_agent,
            &home_path,
        );
        self.run(coroutine)
    }

    /// Creates calendar `id` (display name `name`) under the home-set.
    pub fn create_calendar(
        &mut self,
        id: &str,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<(), WebdavClientError> {
        self.validate_calendar(id)?;

        let home = self.inner.calendar_home_set()?;
        let home_path = home.path().to_string();

        let wire = WireCalendar {
            id: id.to_string(),
            display_name: Some(name.to_string()),
            description: description.map(str::to_string),
            color: color.map(str::to_string),
            ctag: None,
            tz: None,
        };

        let coroutine = WebdavCalendarCreate::new(
            &self.inner.base_url,
            self.inner.auth(),
            &self.inner.user_agent,
            &home_path,
            &wire,
        );
        self.run(coroutine)?;
        Ok(())
    }

    /// Applies `patch` to calendar `id`, merging it against the current
    /// calendar metadata.
    pub fn update_calendar(
        &mut self,
        id: &str,
        patch: CalendarDiff,
    ) -> Result<(), WebdavClientError> {
        self.validate_calendar(id)?;

        let current = self
            .list_calendars()?
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(|| WebdavClientError::CalendarNotFound(id.to_string()))?;

        let display_name = match patch.name {
            Some(name) => Some(name),
            None => Some(current.name),
        };
        let description = match patch.description {
            Some(description) => description,
            None => current.description,
        };
        let color = match patch.color {
            Some(color) => color,
            None => current.color,
        };

        let home = self.inner.calendar_home_set()?;
        let home_path = home.path().to_string();

        let wire = WireCalendar {
            id: id.to_string(),
            display_name,
            description,
            color,
            ctag: None,
            tz: None,
        };

        let coroutine = WebdavCalendarUpdate::new(
            &self.inner.base_url,
            self.inner.auth(),
            &self.inner.user_agent,
            &home_path,
            &wire,
        );
        self.run(coroutine)?;
        Ok(())
    }

    /// Deletes calendar `id`.
    pub fn delete_calendar(&mut self, id: &str) -> Result<(), WebdavClientError> {
        self.validate_calendar(id)?;

        let home = self.inner.calendar_home_set()?;
        let home_path = home.path().to_string();

        let coroutine = WebdavCalendarDelete::new(
            &self.inner.base_url,
            self.inner.auth(),
            &self.inner.user_agent,
            &home_path,
            id,
        );
        self.run(coroutine)?;
        Ok(())
    }

    /// Lists items inside `calendar_id`, applying 1-indexed pagination.
    ///
    /// When `time_range` is set, the server query is constrained to
    /// VEVENT components overlapping the range.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        time_range: Option<&TimeRange>,
    ) -> Result<Vec<CalendarItem>, WebdavClientError> {
        self.validate_calendar(calendar_id)?;

        let home = self.inner.calendar_home_set()?;
        let path = calendar_path(&home, calendar_id);

        let coroutine = WebdavCalendarItemList::new(
            &self.inner.base_url,
            self.inner.auth(),
            &self.inner.user_agent,
            &path,
            calendar_id,
            page,
            page_size,
            time_range,
        );
        self.run(coroutine)
    }

    /// Fetches `item_id` from `calendar_id`.
    pub fn get_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
    ) -> Result<CalendarItem, WebdavClientError> {
        self.validate_calendar(calendar_id)?;
        self.validate_item(item_id)?;

        let home = self.inner.calendar_home_set()?;
        let path = calendar_path(&home, calendar_id);

        let coroutine = WebdavCalendarItemGet::new(
            &self.inner.base_url,
            self.inner.auth(),
            &self.inner.user_agent,
            &path,
            calendar_id,
            item_id,
        );
        self.run(coroutine)
    }

    /// Appends a new item to `calendar_id`. Returns the id the server
    /// confirmed.
    pub fn create_item(
        &mut self,
        calendar_id: &str,
        contents: Vec<u8>,
    ) -> Result<String, WebdavClientError> {
        if contents.is_empty() {
            return Err(WebdavClientError::EmptyItemBody);
        }
        self.validate_calendar(calendar_id)?;

        let id = fresh_item_id()?;

        let home = self.inner.calendar_home_set()?;
        let path = calendar_path(&home, calendar_id);

        let coroutine = WebdavCalendarItemCreate::new(
            &self.inner.base_url,
            self.inner.auth(),
            &self.inner.user_agent,
            &path,
            &id,
            contents,
        );
        self.run(coroutine)
    }

    /// Overwrites `item_id` inside `calendar_id`, gating on `if_match`
    /// when present.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<(), WebdavClientError> {
        if contents.is_empty() {
            return Err(WebdavClientError::EmptyItemBody);
        }
        self.validate_calendar(calendar_id)?;
        self.validate_item(item_id)?;

        let home = self.inner.calendar_home_set()?;
        let path = calendar_path(&home, calendar_id);

        let coroutine = WebdavCalendarItemUpdate::new(
            &self.inner.base_url,
            self.inner.auth(),
            &self.inner.user_agent,
            &path,
            item_id,
            contents,
            if_match,
        );
        self.run(coroutine)?;
        Ok(())
    }

    /// Permanently deletes `item_id` from `calendar_id`.
    pub fn delete_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
    ) -> Result<(), WebdavClientError> {
        self.validate_calendar(calendar_id)?;
        self.validate_item(item_id)?;

        let home = self.inner.calendar_home_set()?;
        let path = calendar_path(&home, calendar_id);

        let coroutine = WebdavCalendarItemDelete::new(
            &self.inner.base_url,
            self.inner.auth(),
            &self.inner.user_agent,
            &path,
            item_id,
        );
        self.run(coroutine)?;
        Ok(())
    }

    /// Rejects an empty calendar id (after trimming surrounding
    /// slashes).
    fn validate_calendar(&self, id: &str) -> Result<(), WebdavClientError> {
        if id.trim_matches('/').is_empty() {
            return Err(WebdavClientError::InvalidCalendar(id.to_string()));
        }
        Ok(())
    }

    /// Rejects an empty item id.
    fn validate_item(&self, id: &str) -> Result<(), WebdavClientError> {
        if id.is_empty() {
            return Err(WebdavClientError::InvalidItemId(id.to_string()));
        }
        Ok(())
    }
}
