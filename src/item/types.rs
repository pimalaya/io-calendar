//! Calendar item shared across all protocols.

#[cfg(feature = "parser")]
use core::str::from_utf8;

use alloc::{string::String, vec::Vec};

#[cfg(feature = "parser")]
use calcard::{Entry, Parser, icalendar::ICalendar};

/// A single calendar item (event, todo, journal entry).
///
/// Strict least-common-denominator shape: contents stay raw iCalendar
/// bytes; the optional `parser` feature exposes calcard-backed helpers
/// on top.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub struct CalendarItem {
    /// Item identifier (file stem for vdir, last URL segment for
    /// CalDAV).
    pub id: String,

    /// Parent calendar identifier.
    pub calendar_id: String,

    /// Entity tag (RFC 9110 section 8.8.3, without surrounding quotes)
    /// when the backend exposes it; vdir surfaces `None`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub etag: Option<String>,

    /// Raw iCalendar bytes.
    pub contents: Vec<u8>,
}

impl CalendarItem {
    /// Returns the raw item bytes.
    pub fn contents(&self) -> &[u8] {
        &self.contents
    }

    /// Parses the bytes as an iCalendar object.
    #[cfg(feature = "parser")]
    pub fn as_ical(&self) -> Option<ICalendar> {
        let text = from_utf8(&self.contents).ok()?;
        match Parser::new(text).entry() {
            Entry::ICalendar(ical) => Some(ical),
            _ => None,
        }
    }
}

/// A CalDAV time-range filter (RFC 4791 section 9.9) passed as a
/// `list_items` option.
///
/// Both bounds are optional: an open-ended range omits the missing
/// bound. Values are UTC timestamps in iCalendar `YYYYMMDDTHHMMSSZ`
/// form. The WebDAV backend pushes this to the server as a `time-range`
/// element; the vdir backend filters fetched items client-side.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeRange {
    start: Option<String>,
    end: Option<String>,
}

impl TimeRange {
    /// Builds a time-range filter from optional `start` / `end` UTC
    /// timestamps (`YYYYMMDDTHHMMSSZ`).
    ///
    /// At least one bound must be present (RFC 4791 section 9.9).
    /// Returns `None` when both are absent or a value does not match
    /// the expected format. Date digits are range-checked (month 01-12,
    /// day 01-31) but not for calendar validity (Feb 30 is accepted).
    pub fn new(start: Option<&str>, end: Option<&str>) -> Option<Self> {
        if start.is_none() && end.is_none() {
            return None;
        }

        if let Some(start) = start {
            if !Self::is_valid_timestamp(start) {
                return None;
            }
        }

        if let Some(end) = end {
            if !Self::is_valid_timestamp(end) {
                return None;
            }
        }

        Some(Self {
            start: start.map(String::from),
            end: end.map(String::from),
        })
    }

    /// Inclusive lower bound, when set.
    pub fn start(&self) -> Option<&str> {
        self.start.as_deref()
    }

    /// Exclusive upper bound, when set.
    pub fn end(&self) -> Option<&str> {
        self.end.as_deref()
    }

    // Validates structural format and basic date ranges. The restricted
    // character set (digits, `T`, `Z` only) also guards against XML
    // injection, since these values are interpolated into the CalDAV
    // request body.
    fn is_valid_timestamp(stamp: &str) -> bool {
        let bytes = stamp.as_bytes();

        if bytes.len() != 16 || bytes[8] != b'T' || bytes[15] != b'Z' {
            return false;
        }

        if !stamp[..8].bytes().all(|b| b.is_ascii_digit())
            || !stamp[9..15].bytes().all(|b| b.is_ascii_digit())
        {
            return false;
        }

        let month: u32 = stamp[4..6].parse().unwrap_or(0);
        let day: u32 = stamp[6..8].parse().unwrap_or(0);
        let hour: u32 = stamp[9..11].parse().unwrap_or(99);
        let minute: u32 = stamp[11..13].parse().unwrap_or(99);
        let second: u32 = stamp[13..15].parse().unwrap_or(99);

        (1..=12).contains(&month)
            && (1..=31).contains(&day)
            && hour <= 23
            && minute <= 59
            && second <= 59
    }
}

/// Kind of a calendar item, derived from the first VCALENDAR child
/// component. Used to filter list output across backends.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum CalendarItemKind {
    /// `VEVENT` component.
    Event,
    /// `VTODO` component.
    Todo,
    /// `VJOURNAL` component.
    Journal,
}
