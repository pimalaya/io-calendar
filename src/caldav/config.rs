use std::borrow::Cow;

use http::Uri;
use secrecy::SecretString;

#[derive(Clone, Debug)]
pub struct CaldavConfig<'a> {
    /// The URI of the Caldav server.
    pub uri: Cow<'a, Uri>,

    /// The authentication/authorization method used to communicate
    /// with the Caldav server.
    pub auth: CaldavAuth<'a>,
}

#[derive(Clone, Debug, Default)]
pub enum CaldavAuth<'a> {
    /// The plain authentication method.
    #[default]
    Plain,

    /// The basic authentication method.
    Basic {
        username: Cow<'a, str>,
        password: Cow<'a, SecretString>,
    },

    /// The bearer authorization method.
    Bearer { token: Cow<'a, SecretString> },
}
