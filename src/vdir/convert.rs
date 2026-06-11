//! Conversions between vdir filesystem types and the shared
//! [`Calendar`] / [`CalendarItem`] types, plus small list
//! helpers shared by the vdir coroutines.

use alloc::{string::ToString, vec::Vec};

use io_vdir::{
    collection::Collection,
    item::{Item as VdirItem, ItemKind as VdirItemKind},
    path::VdirPath,
};

use crate::{calendar::Calendar, item::CalendarItem};

#[cfg(feature = "client")]
impl From<crate::vdir::client::VdirClient> for crate::client::CalendarClientStd {
    fn from(client: crate::vdir::client::VdirClient) -> Self {
        Self::Vdir(client)
    }
}

impl From<Collection> for Calendar {
    fn from(collection: Collection) -> Self {
        let id = collection.id().to_string();
        let name = collection
            .display_name
            .clone()
            .unwrap_or_else(|| id.clone());

        Calendar {
            id,
            name,
            description: collection.description,
            color: collection.color,
            ctag: None,
        }
    }
}

/// Builds the on-disk path of `calendar_id` under `root`. Forwards to
/// [`VdirPath::join`]; performs no filesystem check.
pub(crate) fn calendar_path(root: &VdirPath, calendar_id: &str) -> VdirPath {
    root.join(calendar_id.trim_matches('/'))
}

/// Maps a vdir [`VdirItem`] to a shared [`CalendarItem`], pinning it to
/// `calendar_id`. Returns `None` when the item path has no usable file
/// stem; ETag is `None` because vdir has no entity-tag concept.
pub(crate) fn item_from(item: VdirItem, calendar_id: &str) -> Option<CalendarItem> {
    let id = item.id()?.to_string();

    Some(CalendarItem {
        id,
        calendar_id: calendar_id.to_string(),
        etag: None,
        contents: item.contents,
    })
}

/// Returns `true` when the vdir item kind is an iCalendar object
/// (`.ics`); used to filter vCard items out of the shared calendar
/// API.
pub(crate) fn is_calendar_item(kind: VdirItemKind) -> bool {
    matches!(kind, VdirItemKind::Ical)
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
