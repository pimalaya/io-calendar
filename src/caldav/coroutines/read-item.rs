use calcard::icalendar::ICalendar;
use io_stream::io::StreamIo;

use crate::{
    caldav::{config::CaldavConfig, request::Request},
    item::CalendarItem,
};

use super::send::{Empty, Send, SendError, SendOk, SendResult};

#[derive(Debug)]
pub struct ReadItem {
    calendar_id: Option<String>,
    id: Option<String>,
    send: Send<Empty>,
}

impl ReadItem {
    const BODY: &'static str = "";

    pub fn new(
        config: &CaldavConfig,
        calendar_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Self {
        let calendar_id = calendar_id.as_ref().to_owned();
        let card_id = card_id.as_ref().to_owned();
        let path = &format!("/{calendar_id}/{card_id}.vcf");
        let request = Request::get(config, path);
        let send = Send::new(request, Self::BODY.as_bytes().to_vec());

        Self {
            id: Some(card_id),
            calendar_id: Some(calendar_id),
            send,
        }
    }

    pub fn resume(&mut self, arg: Option<StreamIo>) -> SendResult<CalendarItem> {
        let ok = match self.send.resume(arg) {
            SendResult::Ok(ok) => ok,
            SendResult::Err(err) => return SendResult::Err(err),
            SendResult::Io(io) => return SendResult::Io(io),
        };

        let content = String::from_utf8_lossy(ok.response.body());
        let ical = match ICalendar::parse(content) {
            Ok(ical) => ical,
            Err(err) => return SendResult::Err(SendError::ParseIcalResponseBody(err)),
        };

        let item = CalendarItem {
            id: self.id.take().unwrap(),
            calendar_id: self.calendar_id.take().unwrap(),
            ical,
        };

        SendResult::Ok(SendOk {
            request: ok.request,
            response: ok.response,
            keep_alive: ok.keep_alive,
            body: item,
        })
    }
}
