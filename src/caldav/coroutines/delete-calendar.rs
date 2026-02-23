use io_stream::io::StreamIo;

use crate::caldav::{config::CaldavConfig, request::Request};

use super::send::{Empty, Send, SendResult};

#[derive(Debug)]
pub struct DeleteCalendar(Send<Empty>);

impl DeleteCalendar {
    const BODY: &'static str = "";

    pub fn new(config: &CaldavConfig, id: impl AsRef<str>) -> Self {
        let request = Request::delete(config, id).content_type_xml();
        Self(Send::new(request, Self::BODY.as_bytes().to_vec()))
    }

    pub fn resume(&mut self, arg: Option<StreamIo>) -> SendResult<Empty> {
        self.0.resume(arg)
    }
}
