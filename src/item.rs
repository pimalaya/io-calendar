use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};

use calcard::Entry;
use serde::{Serialize, Serializer};
use thiserror::Error;
use uuid::Uuid;

pub use calcard::icalendar::*;

#[derive(Clone, Debug, Error)]
pub enum ParseCalendarItemError {
    #[error("Invalid iCal format: parsed iCal instead")]
    InvalidFormat,
    #[error("Invalid iCal line: {0}")]
    InvalidLine(String),
    #[error("Unexpected iCal EOF")]
    UnexpectedEof,
    #[error("Too many iCal components")]
    TooManyComponents,
    #[error("Unexpected iCal component end: expected {0:?} got {1:?}")]
    UnexpectedComponentEnd(ICalendarComponentType, ICalendarComponentType),
    #[error("Unterminated iCal component: {0}")]
    UnterminatedComponent(Cow<'static, str>),
    #[error("Unknown iCal error: {0:?}")]
    Unknown(Entry),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CalendarItem {
    pub id: String,
    pub calendar_id: String,
    #[serde(serialize_with = "CalendarItem::serialize_ical")]
    pub ical: ICalendar,
}

impl CalendarItem {
    pub fn new_uuid() -> Uuid {
        Uuid::new_v4()
    }

    pub fn serialize_ical<S: Serializer>(ical: &ICalendar, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&ical.to_string())
    }

    pub fn parse(contents: impl AsRef<str>) -> Result<ICalendar, ParseCalendarItemError> {
        match ICalendar::parse(contents) {
            Ok(ical) => Ok(ical),
            Err(Entry::ICalendar(ical)) => Ok(ical),
            Err(Entry::VCard(_)) => Err(ParseCalendarItemError::InvalidFormat),
            Err(Entry::InvalidLine(line)) => Err(ParseCalendarItemError::InvalidLine(line)),
            Err(Entry::Eof) => Err(ParseCalendarItemError::UnexpectedEof),
            Err(Entry::TooManyComponents) => Err(ParseCalendarItemError::TooManyComponents),
            Err(Entry::UnexpectedComponentEnd { expected, found }) => Err(
                ParseCalendarItemError::UnexpectedComponentEnd(expected, found),
            ),
            Err(Entry::UnterminatedComponent(component)) => {
                Err(ParseCalendarItemError::UnterminatedComponent(component))
            }
            Err(err) => Err(ParseCalendarItemError::Unknown(err)),
        }
    }

    pub fn components(&self) -> impl Iterator<Item = &ICalendarComponent> {
        self.ical.components.iter()
    }
}

impl Hash for CalendarItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.calendar_id.hash(state);
    }
}

impl ToString for CalendarItem {
    fn to_string(&self) -> String {
        self.ical.to_string()
    }
}
