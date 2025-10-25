use io_stream::io::StreamIo;

use crate::{
    caldav::{config::CaldavConfig, request::Request},
    calendar::Calendar,
};

use super::send::{Empty, Send, SendResult};

#[derive(Debug)]
pub struct CreateCalendar(Send<Empty>);

impl CreateCalendar {
    pub fn new(config: &CaldavConfig, mut calendar: Calendar) -> Self {
        let name = match calendar.display_name.take() {
            Some(name) => format!("<displayname>{name}</displayname>"),
            None => String::new(),
        };

        let desc = match &calendar.description.take() {
            Some(desc) => format!("<C:calendar-description>{desc}</C:calendar-description>"),
            None => String::new(),
        };

        let color = match &calendar.color.take() {
            Some(color) => format!("<I:calendar-color>{color}</I:calendar-color>"),
            None => String::new(),
        };

        let request = Request::mkcol(config, calendar.id).content_type_xml();
        let body = format!(include_str!("./create-calendar.xml"), name, color, desc);

        Self(Send::new(request, body.as_bytes().to_vec()))
    }

    pub fn resume(&mut self, arg: Option<StreamIo>) -> SendResult<Empty> {
        self.0.resume(arg)
    }
}
