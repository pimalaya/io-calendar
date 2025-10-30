#![cfg(feature = "caldav")]

use std::{borrow::Cow, env};

use io_calendar::caldav::{
    config::{CaldavAuth, CaldavConfig},
    coroutines::{
        calendar_home_set::CalendarHomeSet, current_user_principal::CurrentUserPrincipal,
        follow_redirects::FollowRedirectsResult, list_calendars::ListCalendars,
        list_items::ListCalendarItems, send::SendResult,
    },
    request::set_uri_path,
};
use io_stream::runtimes::std::handle;
use pimalaya_toolbox::stream::{Stream, Tls};
use secrecy::SecretString;

fn main() {
    env_logger::init();

    let uri = env::var("URI").expect("URI env var").parse().unwrap();

    let username = Cow::Owned(env::var("USERNAME").expect("USERNAME env var"));
    let password = Cow::Owned(SecretString::from(
        env::var("PASSWORD").expect("PASSWORD env var"),
    ));

    println!("connecting to {uri}…");
    let tls = Tls::RustlsRing;
    let mut stream = Stream::connect(&uri, &tls).unwrap();

    let mut config = CaldavConfig {
        uri: Cow::Borrowed(&uri),
        auth: CaldavAuth::Basic { username, password },
    };

    let mut arg = None;
    let mut http = CurrentUserPrincipal::new(&config);

    loop {
        match http.resume(arg.take()) {
            FollowRedirectsResult::Ok(res) => {
                if let Some(uri) = res.body {
                    let uri = if uri.authority().is_some() {
                        uri
                    } else {
                        set_uri_path(config.uri.into_owned(), uri.path())
                    };

                    config.uri = Cow::Owned(uri);

                    if !res.keep_alive {
                        stream = Stream::connect(&config.uri, &tls).unwrap()
                    }
                }

                break;
            }
            FollowRedirectsResult::Err(err) => panic!("{err}"),
            FollowRedirectsResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
            FollowRedirectsResult::Reset(uri) => stream = Stream::connect(&uri, &tls).unwrap(),
        }
    }

    println!("current user principal: {:?}", config.uri);

    let mut arg = None;
    let mut http = CalendarHomeSet::new(&config);

    loop {
        match http.resume(arg.take()) {
            FollowRedirectsResult::Ok(res) => {
                if let Some(uri) = res.body {
                    let uri = if uri.authority().is_some() {
                        uri
                    } else {
                        set_uri_path(config.uri.into_owned(), uri.path())
                    };

                    config.uri = Cow::Owned(uri);

                    if !res.keep_alive {
                        stream = Stream::connect(&config.uri, &tls).unwrap()
                    }
                }

                break;
            }
            FollowRedirectsResult::Err(err) => panic!("{err}"),
            FollowRedirectsResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
            FollowRedirectsResult::Reset(uri) => stream = Stream::connect(&uri, &tls).unwrap(),
        }
    }

    println!("calendar home set: {:?}", config.uri);

    let mut arg = None;
    let mut http = ListCalendars::new(&config);

    let calendars = loop {
        match http.resume(arg.take()) {
            SendResult::Ok(res) => break res.body,
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    println!("calendars: {calendars:#?}");

    let mut arg = None;
    let mut http = ListCalendarItems::new(&config, calendars.into_iter().next().unwrap().id, None);

    let items = loop {
        match http.resume(arg.take()) {
            SendResult::Ok(res) => break res.body,
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    println!("items: {items:#?}");
}
