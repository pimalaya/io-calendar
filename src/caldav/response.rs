use http::{uri::InvalidUri, Uri};
use memchr::memmem;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Multistatus<T> {
    #[serde(rename = "response")]
    pub responses: Option<Vec<PropstatResponse<T>>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MkcolResponse<T> {
    #[serde(rename = "propstat")]
    pub propstats: Option<Vec<Propstat<T>>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PropstatResponse<T> {
    pub href: Value,
    pub status: Option<Status>,
    #[serde(rename = "propstat")]
    pub propstats: Option<Vec<Propstat<T>>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StatusResponse {
    pub status: Status,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Propstat<T> {
    pub prop: T,
    pub status: Status,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HrefProp {
    pub href: Value,
}

impl HrefProp {
    pub fn uri(&self) -> Result<Uri, InvalidUri> {
        self.href.value.parse()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Value {
    #[serde(rename = "$value", default)]
    pub value: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct Status(Value);

impl Status {
    pub fn is_success(&self) -> bool {
        memmem::find(self.0.value.as_bytes(), b" 2").is_some()
    }
}
