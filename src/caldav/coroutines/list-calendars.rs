use std::collections::HashSet;

use io_stream::io::StreamIo;
use log::{debug, trace};
use serde::Deserialize;

use crate::{
    caldav::{config::CaldavConfig, request::Request, response::Multistatus},
    calendar::Calendar,
};

use super::send::{Send, SendOk, SendResult};

#[derive(Debug)]
pub struct ListCalendars(Send<Multistatus<Prop>>);

impl ListCalendars {
    const BODY: &'static str = include_str!("./list-calendars.xml");

    pub fn new(config: &CaldavConfig) -> Self {
        let request = Request::propfind(config, "").content_type_xml().depth(1);
        let body = Self::BODY.as_bytes().into_iter().cloned();
        Self(Send::new(request, body))
    }

    pub fn resume(&mut self, arg: Option<StreamIo>) -> SendResult<HashSet<Calendar>> {
        let ok = match self.0.resume(arg) {
            SendResult::Ok(ok) => ok,
            SendResult::Err(err) => return SendResult::Err(err),
            SendResult::Io(io) => return SendResult::Io(io),
        };

        let mut calendars = HashSet::new();

        let Some(responses) = ok.body.responses else {
            return SendResult::Ok(SendOk {
                request: ok.request,
                response: ok.response,
                keep_alive: ok.keep_alive,
                body: calendars,
            });
        };

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

            let mut is_calendar = None;

            let mut calendar = Calendar {
                id: response
                    .href
                    .value
                    .trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .unwrap() // SAFETY: calendars belong to principal
                    .to_owned(),
                display_name: None,
                description: None,
                color: None,
            };

            for propstat in propstats {
                if let Some(false) = is_calendar {
                    break;
                }

                if !propstat.status.is_success() {
                    debug!("multistatus propstat response error");
                    continue;
                }

                if let Some(rtype) = propstat.prop.resourcetype {
                    if rtype.calendar.is_some() {
                        is_calendar.replace(true);
                    }
                }

                if let Some(name) = propstat.prop.displayname {
                    if !name.trim().is_empty() {
                        calendar.display_name = Some(name);
                    }
                }

                if let Some(desc) = propstat.prop.calendar_description {
                    if !desc.trim().is_empty() {
                        calendar.description = Some(desc);
                    }
                }

                if let Some(color) = propstat.prop.calendar_color {
                    if !color.trim().is_empty() {
                        calendar.color = Some(color);
                    }
                }
            }

            if let Some(true) = is_calendar {
                calendars.insert(calendar);
            }
        }

        SendResult::Ok(SendOk {
            request: ok.request,
            response: ok.response,
            keep_alive: ok.keep_alive,
            body: calendars,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Prop {
    pub resourcetype: Option<ResourceType>,
    pub displayname: Option<String>,
    pub calendar_color: Option<String>,
    pub calendar_description: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ResourceType {
    pub calendar: Option<()>,
}
