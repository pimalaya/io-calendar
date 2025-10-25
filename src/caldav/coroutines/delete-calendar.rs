use io_stream::io::StreamIo;
use serde::Deserialize;

use crate::caldav::{config::CaldavConfig, request::Request, response::StatusResponse};

use super::send::{Send, SendOk, SendResult};

#[derive(Debug)]
pub struct DeleteCalendar(Send<Response>);

impl DeleteCalendar {
    const BODY: &'static str = "";

    pub fn new(config: &CaldavConfig, id: impl AsRef<str>) -> Self {
        let request = Request::delete(config, id).content_type_xml();
        Self(Send::new(request, Self::BODY.as_bytes().to_vec()))
    }

    pub fn resume(&mut self, arg: Option<StreamIo>) -> SendResult<bool> {
        let ok = match self.0.resume(arg) {
            SendResult::Ok(ok) => ok,
            SendResult::Err(err) => return SendResult::Err(err),
            SendResult::Io(io) => return SendResult::Io(io),
        };

        SendResult::Ok(SendOk {
            request: ok.request,
            response: ok.response,
            keep_alive: ok.keep_alive,
            body: ok.body.response.status.is_success(),
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Response {
    pub response: StatusResponse,
}
