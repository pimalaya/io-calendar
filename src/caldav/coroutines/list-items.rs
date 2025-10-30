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
        let calendar_id = calendar_id.as_ref().to_owned();

        let request = Request::report(config, &calendar_id)
            .content_type_xml()
            .depth(1);

        let filter = match filter {
            Some(filter) => format!("<C:comp-filter name=\"{}\" />", filter.as_str()),
            None => String::new(),
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
