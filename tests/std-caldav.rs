#![cfg(feature = "caldav")]

use std::{borrow::Cow, collections::HashSet, net::TcpStream};

use io_calendar::{
    caldav::{
        config::{CaldavAuth, CaldavConfig},
        coroutines::{
            create_calendar::CreateCalendar,
            create_item::CreateCalendarItem,
            delete_calendar::DeleteCalendar,
            delete_item::DeleteCalendarItem,
            list_calendars::ListCalendars,
            list_items::ListCalendarItems,
            send::{SendOk, SendResult},
            update_calendar::UpdateCalendar,
            update_item::UpdateCalendarItem,
        },
    },
    calendar::Calendar,
    item::{CalendarItem, ICalendar},
};
use io_stream::runtimes::std::handle;
use secrecy::SecretString;

#[test]
fn std_caldav() {
    env_logger::init();

    let config = CaldavConfig {
        uri: Cow::Owned("https://127.0.0.1:8001/username".parse().unwrap()),
        auth: CaldavAuth::Basic {
            username: Cow::Borrowed("username"),
            password: Cow::Owned(SecretString::from("password")),
        },
    };

    // should list empty calendars

    let mut stream = TcpStream::connect(config.uri.authority().unwrap().as_str()).unwrap();

    let mut arg = None;
    let mut list = ListCalendars::new(&config);

    let calendars = loop {
        match list.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    assert!(calendars.is_empty());

    // should create calendar without metadata

    let mut calendar = Calendar::new();

    let mut arg = None;
    let mut create = CreateCalendar::new(&config, calendar.clone());

    let _ = loop {
        match create.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    let mut arg = None;
    let mut list = ListCalendars::new(&config);

    let calendars = loop {
        match list.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    assert_eq!(1, calendars.len());

    // should not re-create existing calendar

    let mut arg = None;
    let mut create = CreateCalendar::new(&config, calendar.clone());

    loop {
        match create.resume(arg) {
            SendResult::Ok(_) => panic!("should not be OK"),
            SendResult::Err(_) => break,
            SendResult::Io(io) => match handle(&mut stream, io) {
                Ok(output) => arg = Some(output),
                Err(err) => panic!("{err}"),
            },
        }
    }

    stream = TcpStream::connect(config.uri.authority().unwrap().as_str()).unwrap();

    // should update calendar with metadata

    calendar.display_name = Some("Custom calendar name".into());
    calendar.description = Some("This is a description.".into());
    calendar.color = Some("#000000".into());

    let mut arg = None;
    let mut update = UpdateCalendar::new(&config, calendar.clone());

    let _ = loop {
        match update.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    let mut arg = None;
    let mut list = ListCalendars::new(&config);

    let calendars = loop {
        match list.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    let expected_calendars = HashSet::from_iter([calendar.clone()]);

    assert_eq!(calendars, expected_calendars);

    // should create calendar item

    let mut item = CalendarItem {
        id: CalendarItem::new_uuid().to_string(),
        calendar_id: calendar.id.clone(),
        ical: ICalendar::parse("BEGIN:VCALENDAR\r\nBEGIN:VTODO\r\nUID:abc123\r\nSUMMARY:Test\r\nEND:VTODO\r\nEND:VCALENDAR\r\n").unwrap(),
    };

    let mut arg = None;
    let mut create = CreateCalendarItem::new(&config, item.clone());

    let _ = loop {
        match create.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    let mut arg = None;
    let mut list = ListCalendarItems::new(&config, &calendar.id, None);

    let items = loop {
        match list.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    assert_eq!(items.len(), 1);

    let first_item = items.into_iter().next().unwrap().to_string();

    assert!(first_item.contains("UID:abc123"));
    assert!(first_item.contains("SUMMARY:Test"));

    // should update calendar item

    item.ical = ICalendar::parse("BEGIN:VCALENDAR\r\nBEGIN:VTODO\r\nUID:abc123\r\nSUMMARY:Test2\r\nEND:VTODO\r\nEND:VCALENDAR\r\n").unwrap();

    let mut arg = None;
    let mut update = UpdateCalendarItem::new(&config, item.clone());

    let _ = loop {
        match update.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    let mut arg = None;
    let mut list = ListCalendarItems::new(&config, &calendar.id, None);

    let items = loop {
        match list.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    assert_eq!(items.len(), 1);

    let first_item = items.into_iter().next().unwrap().to_string();

    assert!(first_item.contains("UID:abc123"));
    assert!(first_item.contains("SUMMARY:Test2"));

    // should delete calendar item

    let mut arg = None;
    let mut delete = DeleteCalendarItem::new(&config, &calendar.id, &item.id);

    let _ = loop {
        match delete.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    let mut arg = None;
    let mut list = ListCalendarItems::new(&config, &calendar.id, None);

    let items = loop {
        match list.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    assert_eq!(items.into_iter().count(), 0);

    // should delete calendar

    let mut arg = None;
    let mut delete = DeleteCalendar::new(&config, &calendar.id);

    let _ = loop {
        match delete.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    let mut arg = None;
    let mut list = ListCalendars::new(&config);

    let calendars = loop {
        match list.resume(arg) {
            SendResult::Ok(res) => break handle_http(&config, &mut stream, res),
            SendResult::Err(err) => panic!("{err}"),
            SendResult::Io(io) => arg = Some(handle(&mut stream, io).unwrap()),
        }
    };

    assert!(calendars.is_empty());
}

fn handle_http<T>(config: &CaldavConfig, stream: &mut TcpStream, res: SendOk<T>) -> T {
    if !res.keep_alive {
        *stream = TcpStream::connect(config.uri.authority().unwrap().as_str()).unwrap();
    }

    res.body
}
