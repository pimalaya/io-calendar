//! Std-blocking unified calendar client.
//!
//! [`CalendarClientStd`] is an enum over the single registered backend: a value
//! is exactly one of the compiled-in per-backend clients ([`VdirClient`],
//! [`WebdavClientStd`]). Unlike io-email's multi-backend `EmailClientStd`
//! struct, a calendar account speaks one protocol at a time, so the unified
//! client is an enum rather than a bag of optional slots; dispatch is a plain
//! `match` with no priority order.
//!
//! Build one via the per-backend `From` impls (e.g.
//! `CalendarClientStd::from(VdirClient::new(root))`) or by naming the variant
//! directly.
//!
//! [`VdirClient`]: crate::vdir::client::VdirClient
//! [`WebdavClientStd`]: crate::webdav::client::WebdavClientStd

#[cfg(feature = "webdav")]
use alloc::boxed::Box;
use alloc::{string::String, vec::Vec};

use log::trace;
use thiserror::Error;

use crate::{
    calendar::{Calendar, CalendarDiff},
    item::{CalendarItem, TimeRange},
};

/// Errors surfaced by [`CalendarClientStd`].
///
/// Each variant flattens the registered backend's error type via
/// `#[from]`, so the `?` operator works across the dispatch boundary.
#[derive(Debug, Error)]
pub enum CalendarClientStdError {
    #[cfg(feature = "vdir")]
    #[error(transparent)]
    Vdir(#[from] crate::vdir::client::VdirClientError),
    #[cfg(feature = "webdav")]
    #[error(transparent)]
    Webdav(#[from] crate::webdav::client::WebdavClientError),
}

/// Std-blocking unified calendar client.
///
/// One variant per compiled-in backend; a value always holds exactly
/// one. Each shared-API method dispatches to the active backend's
/// matching method.
#[derive(Debug)]
pub enum CalendarClientStd {
    #[cfg(feature = "vdir")]
    Vdir(crate::vdir::client::VdirClient),
    #[cfg(feature = "webdav")]
    Webdav(Box<crate::webdav::client::WebdavClientStd>),
}

impl CalendarClientStd {
    /// Lists every calendar available to the active account.
    pub fn list_calendars(&mut self) -> Result<Vec<Calendar>, CalendarClientStdError> {
        trace!("list calendars");
        match self {
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => Ok(client.list_calendars()?),
            #[cfg(feature = "webdav")]
            Self::Webdav(client) => Ok(client.list_calendars()?),
        }
    }

    /// Creates calendar `id` (display name `name`), optionally carrying
    /// a description and a color.
    pub fn create_calendar(
        &mut self,
        id: &str,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<(), CalendarClientStdError> {
        trace!("create calendar");
        match self {
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => Ok(client.create_calendar(id, name, description, color)?),
            #[cfg(feature = "webdav")]
            Self::Webdav(client) => Ok(client.create_calendar(id, name, description, color)?),
        }
    }

    /// Applies a partial update to calendar `id`. Fields left as `None`
    /// in `patch` are preserved.
    pub fn update_calendar(
        &mut self,
        id: &str,
        patch: CalendarDiff,
    ) -> Result<(), CalendarClientStdError> {
        trace!("update calendar");
        match self {
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => Ok(client.update_calendar(id, patch)?),
            #[cfg(feature = "webdav")]
            Self::Webdav(client) => Ok(client.update_calendar(id, patch)?),
        }
    }

    /// Deletes calendar `id` and every item it contains.
    pub fn delete_calendar(&mut self, id: &str) -> Result<(), CalendarClientStdError> {
        trace!("delete calendar");
        match self {
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => Ok(client.delete_calendar(id)?),
            #[cfg(feature = "webdav")]
            Self::Webdav(client) => Ok(client.delete_calendar(id)?),
        }
    }

    /// Lists items inside `calendar_id`. `page` is 1-indexed; pass
    /// `None` to default to page 1. `page_size = None` returns the full
    /// window. When `time_range` is set, only VEVENTs overlapping the
    /// range are returned (server-side for WebDAV, client-side for
    /// vdir).
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        time_range: Option<&TimeRange>,
    ) -> Result<Vec<CalendarItem>, CalendarClientStdError> {
        trace!("list items");
        match self {
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => {
                Ok(client.list_items(calendar_id, page, page_size, time_range)?)
            }
            #[cfg(feature = "webdav")]
            Self::Webdav(client) => {
                Ok(client.list_items(calendar_id, page, page_size, time_range)?)
            }
        }
    }

    /// Fetches item `item_id` from `calendar_id`.
    pub fn get_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
    ) -> Result<CalendarItem, CalendarClientStdError> {
        trace!("get item");
        match self {
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => Ok(client.get_item(calendar_id, item_id)?),
            #[cfg(feature = "webdav")]
            Self::Webdav(client) => Ok(client.get_item(calendar_id, item_id)?),
        }
    }

    /// Appends a raw iCalendar item to `calendar_id`. Returns the
    /// identifier the backend assigned to the stored item.
    pub fn create_item(
        &mut self,
        calendar_id: &str,
        contents: Vec<u8>,
    ) -> Result<String, CalendarClientStdError> {
        trace!("create item");
        match self {
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => Ok(client.create_item(calendar_id, contents)?),
            #[cfg(feature = "webdav")]
            Self::Webdav(client) => Ok(client.create_item(calendar_id, contents)?),
        }
    }

    /// Replaces the bytes of `item_id` inside `calendar_id`.
    ///
    /// `if_match` is the backend-specific entity tag to gate the update
    /// on; pass `None` to overwrite unconditionally. Backends without
    /// ETag support (vdir) ignore it.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<(), CalendarClientStdError> {
        trace!("update item");
        match self {
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => {
                Ok(client.update_item(calendar_id, item_id, contents, if_match)?)
            }
            #[cfg(feature = "webdav")]
            Self::Webdav(client) => {
                Ok(client.update_item(calendar_id, item_id, contents, if_match)?)
            }
        }
    }

    /// Permanently deletes `item_id` from `calendar_id`.
    pub fn delete_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
    ) -> Result<(), CalendarClientStdError> {
        trace!("delete item");
        match self {
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => Ok(client.delete_item(calendar_id, item_id)?),
            #[cfg(feature = "webdav")]
            Self::Webdav(client) => Ok(client.delete_item(calendar_id, item_id)?),
        }
    }
}
