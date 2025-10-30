use io_stream::io::StreamIo;
use serde::Deserialize;

use crate::caldav::{config::CaldavConfig, request::Request, response::StatusResponse};

use super::send::{Send, SendOk, SendResult};

#[derive(Debug)]
pub struct DeleteCalendarItem(Send<Option<Response>>);

impl DeleteCalendarItem {
    const BODY: &'static str = "";

    pub fn new(
        config: &CaldavConfig,
        calendar_id: impl AsRef<str>,
        item_id: impl AsRef<str>,
    ) -> Self {
        let calendar_id = calendar_id.as_ref();
        let item_id = item_id.as_ref();
        let path = &format!("/{calendar_id}/{item_id}.ics");
        let request = Request::delete(config, path).content_type_xml();
        Self(Send::new(request, Self::BODY.as_bytes().to_vec()))
    }

    pub fn resume(&mut self, arg: Option<StreamIo>) -> SendResult<bool> {
        let ok = match self.0.resume(arg) {
            SendResult::Ok(ok) => ok,
            SendResult::Err(err) => return SendResult::Err(err),
            SendResult::Io(io) => return SendResult::Io(io),
        };

        let has_no_content = ok.response.status() == 204;

        SendResult::Ok(SendOk {
            request: ok.request,
            response: ok.response,
            keep_alive: ok.keep_alive,
            body: match ok.body {
                Some(body) => body.response.status.is_success(),
                None => has_no_content,
            },
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Response {
    pub response: StatusResponse,
}
