//! Std-blocking Vdir calendar client.
//!
//! Wraps an inner [`io_vdir::client::VdirClient`] (the filesystem root)
//! and pumps io-calendar Vdir coroutines directly against the local
//! filesystem via [`VdirClient::run`]. One shared-API method per
//! operation builds a coroutine and runs it; the inner client stays
//! reachable through [`VdirClient::inner`].

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
    vec,
    vec::Vec,
};
use std::{fs, io};

use getrandom::fill;
use io_vdir::{client::VdirClient as InnerVdirClient, coroutine::*, path::VdirPath};
use log::trace;
use thiserror::Error;

use crate::{
    calendar::{
        Calendar, CalendarDiff,
        vdir::{
            create::{VdirCalendarCreate, VdirCalendarCreateError},
            delete::{VdirCalendarDelete, VdirCalendarDeleteError},
            list::{VdirCalendarList, VdirCalendarListError},
            update::{VdirCalendarUpdate, VdirCalendarUpdateError},
        },
    },
    item::{
        CalendarItem, TimeRange,
        vdir::{
            create::{VdirCalendarItemCreate, VdirCalendarItemCreateError},
            delete::{VdirCalendarItemDelete, VdirCalendarItemDeleteError},
            get::{VdirCalendarItemGet, VdirCalendarItemGetError},
            list::{VdirCalendarItemList, VdirCalendarItemListError},
            update::{VdirCalendarItemUpdate, VdirCalendarItemUpdateError},
        },
    },
    vdir::convert::calendar_path,
};

/// Errors surfaced by [`VdirClient`] while running a coroutine.
///
/// One variant per shared-API Vdir coroutine, plus filesystem and
/// randomness failures from the run loop and the domain validation
/// failures from the client methods.
#[derive(Debug, Error)]
pub enum VdirClientError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Inner(#[from] io_vdir::client::VdirClientError),
    #[error("Failed to gather randomness: {0}")]
    Random(getrandom::Error),
    #[error("Invalid calendar `{0}`")]
    InvalidCalendar(String),
    #[error("Invalid item id `{0}`")]
    InvalidItemId(String),
    #[error("Calendar `{0}` not found")]
    CalendarNotFound(String),
    #[error("Item body is empty")]
    EmptyItemBody,

    #[error(transparent)]
    CalendarCreate(#[from] VdirCalendarCreateError),
    #[error(transparent)]
    CalendarDelete(#[from] VdirCalendarDeleteError),
    #[error(transparent)]
    CalendarList(#[from] VdirCalendarListError),
    #[error(transparent)]
    CalendarUpdate(#[from] VdirCalendarUpdateError),

    #[error(transparent)]
    ItemCreate(#[from] VdirCalendarItemCreateError),
    #[error(transparent)]
    ItemDelete(#[from] VdirCalendarItemDeleteError),
    #[error(transparent)]
    ItemGet(#[from] VdirCalendarItemGetError),
    #[error(transparent)]
    ItemList(#[from] VdirCalendarItemListError),
    #[error(transparent)]
    ItemUpdate(#[from] VdirCalendarItemUpdateError),
}

/// Std-blocking Vdir calendar client built on a filesystem root.
#[derive(Debug)]
pub struct VdirClient {
    pub inner: InnerVdirClient,
}

impl VdirClient {
    /// Builds a client rooted at `root`.
    pub fn new(root: impl Into<VdirPath>) -> Self {
        Self {
            inner: InnerVdirClient::new(root),
        }
    }

    /// Pumps any standard-shape Vdir coroutine (`Yield = VdirYield`,
    /// `Return = Result<T, E>`) against the local filesystem until it
    /// terminates.
    pub fn run<C, T, E>(&self, mut coroutine: C) -> Result<T, VdirClientError>
    where
        C: VdirCoroutine<Yield = VdirYield, Return = Result<T, E>>,
        VdirClientError: From<E>,
    {
        let mut arg: Option<VdirReply> = None;

        loop {
            match coroutine.resume(arg.take()) {
                VdirCoroutineState::Complete(Ok(out)) => return Ok(out),
                VdirCoroutineState::Complete(Err(err)) => return Err(err.into()),
                VdirCoroutineState::Yielded(VdirYield::WantsRandom { len }) => {
                    let mut bytes = vec![0u8; len];
                    fill(&mut bytes).map_err(VdirClientError::Random)?;
                    arg = Some(VdirReply::Random(bytes));
                }
                VdirCoroutineState::Yielded(VdirYield::WantsFileExists(paths)) => {
                    let mut out = BTreeMap::new();
                    for path in paths {
                        let exists = fs::metadata(path.as_str())
                            .map(|m| m.is_file())
                            .unwrap_or(false);
                        trace!("file_exists {path}: {exists}");
                        out.insert(path, exists);
                    }
                    arg = Some(VdirReply::FileExists(out));
                }
                VdirCoroutineState::Yielded(VdirYield::WantsDirExists(paths)) => {
                    let mut out = BTreeMap::new();
                    for path in paths {
                        let exists = fs::metadata(path.as_str())
                            .map(|m| m.is_dir())
                            .unwrap_or(false);
                        trace!("dir_exists {path}: {exists}");
                        out.insert(path, exists);
                    }
                    arg = Some(VdirReply::DirExists(out));
                }
                VdirCoroutineState::Yielded(VdirYield::WantsDirRead(paths)) => {
                    let mut entries = BTreeMap::new();
                    for path in paths {
                        trace!("read_dir {path}");
                        let mut names = BTreeSet::new();
                        match fs::read_dir(path.as_str()) {
                            Ok(iter) => {
                                for entry in iter {
                                    let entry = entry?;
                                    names.insert(normalize_path(entry.path()));
                                }
                            }
                            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                            Err(err) => return Err(err.into()),
                        }
                        entries.insert(path, names);
                    }
                    arg = Some(VdirReply::DirRead(entries));
                }
                VdirCoroutineState::Yielded(VdirYield::WantsFileRead(paths)) => {
                    let mut contents = BTreeMap::new();
                    for path in paths {
                        trace!("read_file {path}");
                        let bytes = fs::read(path.as_str())?;
                        contents.insert(path, bytes);
                    }
                    arg = Some(VdirReply::FileRead(contents));
                }
                VdirCoroutineState::Yielded(VdirYield::WantsFileCreate(files)) => {
                    for (path, bytes) in files {
                        trace!("write {path} ({} bytes)", bytes.len());
                        if let Some(parent) = std::path::Path::new(path.as_str()).parent() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::write(path.as_str(), &bytes)?;
                    }
                    arg = Some(VdirReply::FileCreate);
                }
                VdirCoroutineState::Yielded(VdirYield::WantsDirCreate(paths)) => {
                    for path in paths {
                        trace!("create_dir_all {path}");
                        fs::create_dir_all(path.as_str())?;
                    }
                    arg = Some(VdirReply::DirCreate);
                }
                VdirCoroutineState::Yielded(VdirYield::WantsDirRemove(paths)) => {
                    for path in paths {
                        trace!("remove_dir_all {path}");
                        fs::remove_dir_all(path.as_str())?;
                    }
                    arg = Some(VdirReply::DirRemove);
                }
                VdirCoroutineState::Yielded(VdirYield::WantsFileRemove(paths)) => {
                    for path in paths {
                        trace!("remove_file {path}");
                        fs::remove_file(path.as_str())?;
                    }
                    arg = Some(VdirReply::FileRemove);
                }
                VdirCoroutineState::Yielded(VdirYield::WantsRename(pairs)) => {
                    for (from, to) in pairs {
                        trace!("rename {from} -> {to}");
                        fs::rename(from.as_str(), to.as_str())?;
                    }
                    arg = Some(VdirReply::Rename);
                }
                VdirCoroutineState::Yielded(VdirYield::WantsCopy(pairs)) => {
                    for (from, to) in pairs {
                        trace!("copy {from} -> {to}");
                        fs::copy(from.as_str(), to.as_str())?;
                    }
                    arg = Some(VdirReply::Copy);
                }
            }
        }
    }

    /// Lists every calendar under the configured root, sorted by name.
    pub fn list_calendars(&self) -> Result<Vec<Calendar>, VdirClientError> {
        self.run(VdirCalendarList::new(self.inner.root().clone()))
    }

    /// Creates calendar `id` (display name `name`) under the root.
    pub fn create_calendar(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<(), VdirClientError> {
        self.validate_calendar(id)?;
        self.run(VdirCalendarCreate::new(
            self.inner.root(),
            id,
            name,
            description,
            color,
        )?)
    }

    /// Applies `patch` to calendar `id`, merging it against the current
    /// collection metadata.
    pub fn update_calendar(&self, id: &str, patch: CalendarDiff) -> Result<(), VdirClientError> {
        self.validate_calendar(id)?;

        let collections = self.inner.list_collections()?;
        let current = collections
            .into_iter()
            .find(|c| c.id() == id)
            .ok_or_else(|| VdirClientError::CalendarNotFound(id.to_string()))?;

        let name = match patch.name {
            Some(name) => name,
            None => current.display_name.unwrap_or_else(|| id.to_string()),
        };
        let description = match patch.description {
            Some(description) => description,
            None => current.description,
        };
        let color = match patch.color {
            Some(color) => color,
            None => current.color,
        };

        self.run(VdirCalendarUpdate::new(
            self.inner.root(),
            id,
            name,
            description,
            color,
        ))
    }

    /// Recursively removes calendar `id`.
    pub fn delete_calendar(&self, id: &str) -> Result<(), VdirClientError> {
        self.validate_calendar(id)?;
        self.run(VdirCalendarDelete::new(self.inner.root(), id))
    }

    /// Lists items inside `calendar_id`, applying 1-indexed pagination.
    ///
    /// When `time_range` is set, the fetched items are filtered
    /// client-side, keeping only VEVENTs whose start date falls in the
    /// range (the filesystem backend has no server-side query). Needs
    /// the `parser` feature; without it the range is ignored.
    pub fn list_items(
        &self,
        calendar_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        time_range: Option<&TimeRange>,
    ) -> Result<Vec<CalendarItem>, VdirClientError> {
        self.validate_calendar(calendar_id)?;
        let path = calendar_path(self.inner.root(), calendar_id);
        let items = self.run(VdirCalendarItemList::new(
            path,
            calendar_id,
            page,
            page_size,
        ))?;
        Ok(filter_time_range(items, time_range))
    }

    /// Fetches `item_id` from `calendar_id`.
    pub fn get_item(
        &self,
        calendar_id: &str,
        item_id: &str,
    ) -> Result<CalendarItem, VdirClientError> {
        self.validate_calendar(calendar_id)?;
        self.validate_item(item_id)?;
        let path = calendar_path(self.inner.root(), calendar_id);
        self.run(VdirCalendarItemGet::new(path, calendar_id, item_id))
    }

    /// Appends a new iCalendar item to `calendar_id`. Returns its
    /// assigned id.
    pub fn create_item(
        &self,
        calendar_id: &str,
        contents: Vec<u8>,
    ) -> Result<String, VdirClientError> {
        if contents.is_empty() {
            return Err(VdirClientError::EmptyItemBody);
        }
        self.validate_calendar(calendar_id)?;
        let path = calendar_path(self.inner.root(), calendar_id);
        self.run(VdirCalendarItemCreate::new(path, contents)?)
    }

    /// Overwrites `item_id` inside `calendar_id`. `if_match` is ignored:
    /// vdir has no entity-tag concept.
    pub fn update_item(
        &self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        _if_match: Option<&str>,
    ) -> Result<(), VdirClientError> {
        if contents.is_empty() {
            return Err(VdirClientError::EmptyItemBody);
        }
        self.validate_calendar(calendar_id)?;
        self.validate_item(item_id)?;
        let path = calendar_path(self.inner.root(), calendar_id);
        self.run(VdirCalendarItemUpdate::new(path, item_id, contents)?)
    }

    /// Permanently deletes `item_id` from `calendar_id`.
    pub fn delete_item(&self, calendar_id: &str, item_id: &str) -> Result<(), VdirClientError> {
        self.validate_calendar(calendar_id)?;
        self.validate_item(item_id)?;
        let path = calendar_path(self.inner.root(), calendar_id);
        self.run(VdirCalendarItemDelete::new(path, item_id))
    }

    /// Rejects an empty calendar id (after trimming surrounding
    /// slashes).
    fn validate_calendar(&self, id: &str) -> Result<(), VdirClientError> {
        if id.trim_matches('/').is_empty() {
            return Err(VdirClientError::InvalidCalendar(id.to_string()));
        }
        Ok(())
    }

    /// Rejects an empty item id.
    fn validate_item(&self, id: &str) -> Result<(), VdirClientError> {
        if id.is_empty() {
            return Err(VdirClientError::InvalidItemId(id.to_string()));
        }
        Ok(())
    }
}

/// Normalizes a host [`std::path::PathBuf`] into a `/`-separated
/// [`VdirPath`].
fn normalize_path(path: std::path::PathBuf) -> VdirPath {
    let s = path.to_string_lossy().into_owned();
    #[cfg(windows)]
    let s = s.replace('\\', "/");
    VdirPath::new(s)
}

/// Keeps only the items matching `time_range`, when set: VEVENTs whose
/// `DTSTART` date is within `[start, end)` at day precision.
#[cfg(feature = "parser")]
fn filter_time_range(
    items: Vec<CalendarItem>,
    time_range: Option<&TimeRange>,
) -> Vec<CalendarItem> {
    let Some(range) = time_range else {
        return items;
    };

    items
        .into_iter()
        .filter(|item| event_in_range(item, range))
        .collect()
}

/// Without the `parser` feature the items cannot be inspected, so the
/// range is ignored and every fetched item is returned.
#[cfg(not(feature = "parser"))]
fn filter_time_range(
    items: Vec<CalendarItem>,
    time_range: Option<&TimeRange>,
) -> Vec<CalendarItem> {
    if time_range.is_some() {
        trace!("vdir time-range filter ignored: parser feature is disabled");
    }
    items
}

/// Whether `item`'s first VEVENT carries a `DTSTART` date inside
/// `range` (inclusive lower bound, exclusive upper bound, day
/// precision). Items without a parseable VEVENT start are dropped.
#[cfg(feature = "parser")]
fn event_in_range(item: &CalendarItem, range: &TimeRange) -> bool {
    use alloc::format;

    use calcard::icalendar::{ICalendarComponentType, ICalendarProperty, ICalendarValue};

    let Some(ical) = item.as_ical() else {
        return false;
    };

    let Some(vevent) = ical
        .components
        .iter()
        .find(|component| component.component_type == ICalendarComponentType::VEvent)
    else {
        return false;
    };

    let Some(property) = vevent.property(&ICalendarProperty::Dtstart) else {
        return false;
    };

    let date = property.values.iter().find_map(|value| match value {
        ICalendarValue::PartialDateTime(pdt) => match (pdt.year, pdt.month, pdt.day) {
            (Some(year), Some(month), Some(day)) => Some(format!("{year:04}{month:02}{day:02}")),
            _ => None,
        },
        _ => None,
    });

    let Some(date) = date else {
        return false;
    };

    if let Some(start) = range.start() {
        if date.as_str() < &start[..8] {
            return false;
        }
    }

    if let Some(end) = range.end() {
        if date.as_str() >= &end[..8] {
            return false;
        }
    }

    true
}
