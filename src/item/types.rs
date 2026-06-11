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
