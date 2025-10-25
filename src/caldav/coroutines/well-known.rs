use http::{header::LOCATION, Method, StatusCode, Uri};
use io_http::v1_1::coroutines::send::{SendHttp, SendHttpError, SendHttpResult};
use io_stream::io::StreamIo;
use thiserror::Error;

use crate::caldav::{config::CaldavConfig, request::Request};

#[derive(Debug)]
pub struct WellKnownOk {
    pub uri: Uri,
    pub keep_alive: bool,
}

#[derive(Debug, Error)]
pub enum WellKnownError {
    #[error("Expected a well known redirection, got {0}: {1}")]
    NotRedirected(StatusCode, String),
    #[error("Missing redirect location in HTTP response")]
    MissingLocationHeader,
    #[error("Invalid redirect location in HTTP response: {0}")]
    InvalidLocationHeader(#[source] http::header::ToStrError, String),
    #[error("Invalid redirect location in HTTP response: {0}")]
    InvalidLocationUri(#[source] http::uri::InvalidUri, String),

    #[error(transparent)]
    Send(#[from] SendHttpError),
}

/// Send result returned by the coroutine's resume function.
#[derive(Debug)]
pub enum WellKnownResult {
    /// The coroutine has successfully terminated its execution.
    Ok(WellKnownOk),
    /// The coroutine encountered an error.
    Err(WellKnownError),
    /// The coroutine wants stream I/O.
    Io(StreamIo),
}

#[derive(Debug)]
pub struct WellKnown(SendHttp);

impl WellKnown {
    pub fn new(config: &CaldavConfig, method: Option<Method>) -> Self {
        let method = method.unwrap_or(Method::GET);
        let request = Request::new(config, method, "");
        Self(SendHttp::new(request.body([])))
    }

    pub fn resume(&mut self, arg: Option<StreamIo>) -> WellKnownResult {
        let ok = match self.0.resume(arg) {
            SendHttpResult::Ok(ok) => ok,
            SendHttpResult::Err(err) => return WellKnownResult::Err(err.into()),
            SendHttpResult::Io(io) => return WellKnownResult::Io(io),
        };

        let status = ok.response.status();

        if !status.is_redirection() {
            let body = String::from_utf8_lossy(ok.response.body()).to_string();
            return WellKnownResult::Err(WellKnownError::NotRedirected(status, body));
        }

        let Some(uri) = ok.response.headers().get(LOCATION) else {
            return WellKnownResult::Err(WellKnownError::MissingLocationHeader);
        };

        let uri = match uri.to_str() {
            Ok(uri) => uri,
            Err(err) => {
                let err = WellKnownError::InvalidLocationHeader(err, format!("{uri:?}"));
                return WellKnownResult::Err(err);
            }
        };

        let uri: Uri = match uri.parse() {
            Ok(uri) => uri,
            Err(err) => {
                let err = WellKnownError::InvalidLocationUri(err, uri.to_string());
                return WellKnownResult::Err(err);
            }
        };

        let same_scheme = if let Some(scheme) = uri.scheme() {
            ok.request.uri().scheme() == Some(scheme)
        } else {
            true
        };

        let same_authority = if let Some(auth) = uri.authority() {
            ok.request.uri().authority() == Some(auth)
        } else {
            true
        };

        let keep_alive = ok.keep_alive && same_scheme && same_authority;

        WellKnownResult::Ok(WellKnownOk { uri, keep_alive })
    }
}
