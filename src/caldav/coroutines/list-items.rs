use std::collections::HashSet;

use calcard::icalendar::{ICalendar, ICalendarComponentType};
use io_stream::io::StreamIo;
use log::{debug, trace};
use serde::Deserialize;

use crate::{
    caldav::{
        config::CaldavConfig,
        request::Request,
        response::{Multistatus, Value},
    },
    item::CalendarItem,
};

use super::send::{Send, SendOk, SendResult};

/// A CalDAV time-range filter (RFC 4791 §9.9).
///
/// Both fields are optional per the RFC (`#IMPLIED`). An open-ended range
/// omits the missing bound, letting the server handle it natively.
///
/// Values are UTC timestamps in iCalendar format: `YYYYMMDDTHHMMSSZ`.
#[derive(Clone, Debug)]
pub struct TimeRange {
    start: Option<String>,
    end: Option<String>,
}

impl TimeRange {
    /// Create a new time-range filter.
    ///
    /// Values must be UTC timestamps in iCalendar format (`YYYYMMDDTHHMMSSZ`).
    /// Returns `None` if a value doesn't match the expected format.
    pub fn new(start: Option<&str>, end: Option<&str>) -> Option<Self> {
        if let Some(s) = start {
            if !Self::is_valid_timestamp(s) {
                return None;
            }
        }
        if let Some(e) = end {
            if !Self::is_valid_timestamp(e) {
                return None;
            }
        }
        Some(Self {
            start: start.map(String::from),
            end: end.map(String::from),
        })
    }

    fn is_valid_timestamp(s: &str) -> bool {
        s.len() == 16
            && s.as_bytes()[8] == b'T'
            && s.as_bytes()[15] == b'Z'
            && s[..8].bytes().all(|b| b.is_ascii_digit())
            && s[9..15].bytes().all(|b| b.is_ascii_digit())
    }

    pub fn start(&self) -> Option<&str> {
        self.start.as_deref()
    }

    pub fn end(&self) -> Option<&str> {
        self.end.as_deref()
    }
}

#[derive(Debug)]
pub struct ListCalendarItems {
    calendar_id: String,
    send: Send<Multistatus<Prop>>,
}

impl ListCalendarItems {
    pub fn new(
        config: &CaldavConfig,
        calendar_id: impl AsRef<str>,
        filter: Option<ICalendarComponentType>,
    ) -> Self {
        Self::with_time_range(config, calendar_id, filter, None)
    }

    pub fn with_time_range(
        config: &CaldavConfig,
        calendar_id: impl AsRef<str>,
        filter: Option<ICalendarComponentType>,
        time_range: Option<&TimeRange>,
    ) -> Self {
        let calendar_id = calendar_id.as_ref().to_owned();

        let request = Request::report(config, &calendar_id)
            .content_type_xml()
            .depth(1);

        let filter = match (filter, time_range) {
            (Some(f), Some(tr)) => {
                let mut attrs = String::new();
                if let Some(s) = tr.start() {
                    attrs.push_str(&format!(" start=\"{s}\""));
                }
                if let Some(e) = tr.end() {
                    attrs.push_str(&format!(" end=\"{e}\""));
                }
                format!(
                    "<C:comp-filter name=\"{}\">\
                       <C:time-range{} />\
                     </C:comp-filter>",
                    f.as_str(),
                    attrs,
                )
            }
            (Some(f), None) => format!("<C:comp-filter name=\"{}\" />", f.as_str()),
            (None, Some(_)) => {
                debug!("time_range ignored: a comp-filter is required for time-range filtering");
                String::new()
            }
            (None, None) => String::new(),
        };

        let body = format!(include_str!("./list-items.xml"), filter);

        Self {
            calendar_id,
            send: Send::new(request, body.as_bytes().to_vec()),
        }
    }

    pub fn resume(&mut self, arg: Option<StreamIo>) -> SendResult<HashSet<CalendarItem>> {
        let ok = match self.send.resume(arg) {
            SendResult::Ok(ok) => ok,
            SendResult::Err(err) => return SendResult::Err(err),
            SendResult::Io(io) => return SendResult::Io(io),
        };

        let mut items = HashSet::new();

        if let Some(responses) = ok.body.responses {
            for response in responses {
                trace!("process multistatus");

                if let Some(status) = response.status {
                    if !status.is_success() {
                        debug!("multistatus response error");
                        continue;
                    }
                };

                let Some(propstats) = response.propstats else {
                    continue;
                };

                let id = response
                    .href
                    .value
                    .trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .unwrap() // SAFETY: calendars belong to principal
                    .trim_end_matches(".ics")
                    .to_owned();

                let mut item = None;

                for propstat in propstats {
                    if !propstat.status.is_success() {
                        debug!("multistatus propstat error");
                        continue;
                    }

                    let Some(content) = propstat.prop.calendar_data else {
                        continue;
                    };

                    let Ok(ical) = ICalendar::parse(content.value) else {
                        continue;
                    };

                    item.replace(CalendarItem {
                        id: id.to_string(),
                        calendar_id: self.calendar_id.clone(),
                        ical,
                    });

                    break;
                }

                let Some(item) = item else {
                    continue;
                };

                items.insert(item);
            }
        };

        SendResult::Ok(SendOk {
            request: ok.request,
            response: ok.response,
            keep_alive: ok.keep_alive,
            body: items,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Prop {
    pub calendar_data: Option<Value>,
}
