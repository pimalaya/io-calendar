//! Conversions between WebDAV wire types and the shared
//! [`Calendar`] / [`CalendarItem`] types, plus small helpers
//! shared by the WebDAV coroutines and the `From` impl that wraps an
//! already-built [`WebdavClientStd`] into the unified client's WebDAV
//! variant.
//!
//! [`WebdavClientStd`]: crate::webdav::client::WebdavClientStd

use alloc::{string::ToString, vec::Vec};

#[cfg(feature = "client")]
use alloc::{format, string::String};

use io_webdav::rfc4791::{
    calendar::Calendar as WireCalendar,
    item::{ItemBody, ItemEntry},
};
#[cfg(feature = "client")]
use url::Url;

use crate::{calendar::Calendar, item::CalendarItem};

#[cfg(feature = "client")]
impl From<crate::webdav::client::WebdavClientStd> for crate::client::CalendarClientStd {
    fn from(client: crate::webdav::client::WebdavClientStd) -> Self {
        Self::Webdav(alloc::boxed::Box::new(client))
    }
}

impl From<WireCalendar> for Calendar {
    fn from(wire: WireCalendar) -> Self {
        let id = wire.id;
        let name = wire.display_name.clone().unwrap_or_else(|| id.clone());

        Calendar {
            id,
            name,
            description: wire.description,
            color: wire.color,
            ctag: wire.ctag,
        }
    }
}

/// Maps a WebDAV [`ItemEntry`] to a shared [`CalendarItem`], pinning it
/// to `calendar_id`.
pub(crate) fn item_from_entry(entry: ItemEntry, calendar_id: &str) -> CalendarItem {
    CalendarItem {
        id: entry.id,
        calendar_id: calendar_id.to_string(),
        etag: entry.etag,
        contents: entry.data,
    }
}

/// Maps a WebDAV [`ItemBody`] to a shared [`CalendarItem`]. The body
/// carries no id, so the requested `item_id` is used.
pub(crate) fn item_from_body(body: ItemBody, calendar_id: &str, item_id: &str) -> CalendarItem {
    CalendarItem {
        id: item_id.to_string(),
        calendar_id: calendar_id.to_string(),
        etag: body.etag,
        contents: body.data,
    }
}

/// Builds the collection path of `calendar_id` under the home-set URL
/// (trim the home-set trailing slash and the id's surrounding slashes).
#[cfg(feature = "client")]
pub(crate) fn calendar_path(home: &Url, calendar_id: &str) -> String {
    let base = home.path().trim_end_matches('/');
    let id = calendar_id.trim_matches('/');
    format!("{base}/{id}")
}

/// 1-indexed pagination on an in-memory list. `page_size = None`
/// returns the full slice; `page_size = 0` or a page past the end
/// returns an empty vector.
pub(crate) fn paginate<T>(items: Vec<T>, page: Option<u32>, page_size: Option<u32>) -> Vec<T> {
    let Some(size) = page_size else {
        return items;
    };

    if size == 0 {
        return Vec::new();
    }

    let page = page.unwrap_or(1).max(1);
    let skip = ((page - 1) as usize).saturating_mul(size as usize);

    if skip >= items.len() {
        return Vec::new();
    }

    items.into_iter().skip(skip).take(size as usize).collect()
}

/// Generates a fresh RFC 4122 v4 item id from the system entropy
/// source.
///
/// CalDAV requires the caller to supply the resource name; the
/// iCalendar UID parsing path is gated behind the optional parser
/// feature upstream, so on the bare client API we always synthesize the
/// id.
#[cfg(feature = "client")]
pub(crate) fn fresh_item_id() -> Result<String, crate::webdav::client::WebdavClientError> {
    use crate::webdav::client::WebdavClientError;

    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)
        .map_err(|_| WebdavClientError::OperationFailed("gather randomness"))?;

    // NOTE: RFC 4122 4.4 stamps version 4 and variant 10xx.
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = [0u8; 36];
    let mut cursor = 0;
    for (i, byte) in bytes.iter().enumerate() {
        if matches!(i, 4 | 6 | 8 | 10) {
            out[cursor] = b'-';
            cursor += 1;
        }
        out[cursor] = HEX[(byte >> 4) as usize];
        out[cursor + 1] = HEX[(byte & 0x0f) as usize];
        cursor += 2;
    }

    Ok(String::from_utf8(out.to_vec()).expect("ASCII hex is always valid UTF-8"))
}
