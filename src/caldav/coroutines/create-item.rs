use io_stream::io::StreamIo;

use crate::{
    caldav::{config::CaldavConfig, request::Request},
    item::CalendarItem,
};

use super::send::{Empty, Send, SendResult};

#[derive(Debug)]
pub struct CreateCalendarItem(Send<Empty>);

impl CreateCalendarItem {
    pub fn new(config: &CaldavConfig, item: CalendarItem) -> Self {
        let path = format!("/{}/{}.vcf", item.calendar_id, item.id);
        let request = Request::put(config, path).content_type_vcard();
        let body = item.to_string().into_bytes();
        Self(Send::new(request, body))
    }

    pub fn resume(&mut self, arg: Option<StreamIo>) -> SendResult<Empty> {
        self.0.resume(arg)
    }
}
