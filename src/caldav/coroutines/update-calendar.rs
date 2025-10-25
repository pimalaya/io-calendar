use io_stream::io::StreamIo;
use log::{debug, trace};
use serde::Deserialize;

use crate::{
    caldav::{config::CaldavConfig, request::Request, response::MkcolResponse},
    calendar::Calendar,
};

use super::send::{Send, SendOk, SendResult};

#[derive(Debug)]
pub struct UpdateCalendar(Send<MkcolResponse<Prop>>);

impl UpdateCalendar {
    pub fn new(config: &CaldavConfig, mut calendar: Calendar) -> Self {
        let name = match calendar.display_name.take() {
            Some(name) => format!("<displayname>{name}</displayname>"),
            None => String::new(),
        };

        let color = match calendar.color.take() {
            Some(color) => format!("<I:calendar-color>{color}</I:calendar-color>"),
            None => String::new(),
        };

        let desc = match calendar.description.take() {
            Some(desc) => format!("<C:calendar-description>{desc}</C:calendar-description>"),
            None => String::new(),
        };

        let request = Request::proppatch(config, calendar.id).content_type_xml();
        let body = format!(include_str!("./update-calendar.xml"), name, color, desc);

        Self(Send::new(request, body.as_bytes().to_vec()))
    }

    pub fn resume(&mut self, arg: Option<StreamIo>) -> SendResult<()> {
        let ok = match self.0.resume(arg) {
            SendResult::Ok(ok) => ok,
            SendResult::Err(err) => return SendResult::Err(err),
            SendResult::Io(io) => return SendResult::Io(io),
        };

        if let Some(propstats) = ok.body.propstats {
            for propstat in propstats {
                if !propstat.status.is_success() {
                    debug!("multistatus propstat error");
                    continue;
                }

                match propstat.prop.displayname {
                    Some(name) => trace!("calendar displayname successfully created: {name}"),
                    None => debug!("adressbook displayname could not be created"),
                }

                match propstat.prop.calendar_description {
                    Some(desc) => trace!("calendar description successfully created: {desc}"),
                    None => debug!("calendar description could not be created"),
                }

                match propstat.prop.calendar_color {
                    Some(color) => trace!("calendar color successfully created: {color}"),
                    None => debug!("calendar color could not be created"),
                }
            }
        }

        SendResult::Ok(SendOk {
            request: ok.request,
            response: ok.response,
            keep_alive: ok.keep_alive,
            body: (),
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Prop {
    pub displayname: Option<String>,
    pub calendar_color: Option<String>,
    pub calendar_description: Option<String>,
}
