#![cfg(feature = "vdir")]

use std::{collections::HashSet, io::ErrorKind};

use io_calendar::{
    calendar::Calendar,
    item::{CalendarItem, ICalendar},
    vdir::coroutines::{
        create_calendar::{CreateCalendar, CreateCalendarResult},
        create_item::{CreateCalendarItem, CreateCalendarItemResult},
        delete_calendar::{DeleteCalendar, DeleteCalendarResult},
        delete_item::{DeleteCalendarItem, DeleteCalendarItemResult},
        list_calendars::{ListCalendars, ListCalendarsResult},
        list_items::{ListCalendarItems, ListCalendarItemsResult},
        update_calendar::{UpdateCalendar, UpdateCalendarResult},
        update_item::{UpdateCalendarItem, UpdateCalendarItemResult},
    },
};
use io_fs::runtimes::std::handle;
use tempdir::TempDir;

#[test]
fn std_vdir() {
    env_logger::init();

    let workdir = TempDir::new("test-vdir-std").unwrap();
    let root = workdir.path();

    // should list empty calendars

    let mut arg = None;
    let mut list = ListCalendars::new(&root);

    let calendars = loop {
        match list.resume(arg) {
            ListCalendarsResult::Ok(calendars) => break calendars,
            ListCalendarsResult::Err(err) => panic!("{err}"),
            ListCalendarsResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    };

    assert!(calendars.is_empty());

    // should create calendar without metadata

    let mut calendar = Calendar::new();

    let mut arg = None;
    let mut create = CreateCalendar::new(root, calendar.clone());

    loop {
        match create.resume(arg) {
            CreateCalendarResult::Ok => break,
            CreateCalendarResult::Err(err) => panic!("{err}"),
            CreateCalendarResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    }

    let mut arg = None;
    let mut list = ListCalendars::new(&root);

    let calendars = loop {
        match list.resume(arg) {
            ListCalendarsResult::Ok(calendars) => break calendars,
            ListCalendarsResult::Err(err) => panic!("{err}"),
            ListCalendarsResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    };

    let expected_calendars = HashSet::from_iter([calendar.clone()]);

    assert_eq!(calendars, expected_calendars);

    // should not re-create existing calendar

    let mut arg = None;
    let mut create = CreateCalendar::new(root, calendar.clone());

    loop {
        match create.resume(arg) {
            CreateCalendarResult::Ok => panic!("should not be OK"),
            CreateCalendarResult::Err(err) => panic!("{err}"),
            CreateCalendarResult::Io(io) => match handle(io) {
                Ok(output) => arg = Some(output),
                Err(err) => break assert_eq!(err.kind(), ErrorKind::AlreadyExists),
            },
        }
    }

    // should update calendar with metadata

    calendar.display_name = Some("Custom calendar name".into());
    calendar.description = Some("This is a description.".into());
    calendar.color = Some("#000000".into());

    let mut arg = None;
    let mut update = UpdateCalendar::new(root, calendar.clone());

    loop {
        match update.resume(arg) {
            UpdateCalendarResult::Ok => break,
            UpdateCalendarResult::Err(err) => panic!("{err}"),
            UpdateCalendarResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    }

    let mut arg = None;
    let mut list = ListCalendars::new(&root);

    let calendars = loop {
        match list.resume(arg) {
            ListCalendarsResult::Ok(calendars) => break calendars,
            ListCalendarsResult::Err(err) => panic!("{err}"),
            ListCalendarsResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    };

    let expected_calendars = HashSet::from_iter([calendar.clone()]);

    assert_eq!(calendars, expected_calendars);

    // should create calendar item

    let mut item = CalendarItem {
        id: CalendarItem::new_uuid().to_string(),
        calendar_id: calendar.id.clone(),
        ical: ICalendar::parse("BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nDTSTART:19970714T170000Z\r\nSUMMARY:Test\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n").unwrap(),
    };

    let mut arg = None;
    let mut create = CreateCalendarItem::new(root, item.clone());

    loop {
        match create.resume(arg) {
            CreateCalendarItemResult::Ok => break,
            CreateCalendarItemResult::Err(err) => panic!("{err}"),
            CreateCalendarItemResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    }

    let mut arg = None;
    let mut list = ListCalendarItems::new(root, &calendar.id);

    let items = loop {
        match list.resume(arg) {
            ListCalendarItemsResult::Ok(items) => break items,
            ListCalendarItemsResult::Err(err) => panic!("{err}"),
            ListCalendarItemsResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    };

    assert_eq!(items.len(), 1);

    let first_item = items.into_iter().next().unwrap();

    assert_eq!(
        first_item.to_string(),
        "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nDTSTART:19970714T170000Z\r\nSUMMARY:Test\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    );

    // should update calendar item

    item.ical = ICalendar::parse("BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nDTSTART:19970714T170000Z\r\nSUMMARY:Test2\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n").unwrap();

    let mut arg = None;
    let mut update = UpdateCalendarItem::new(root, item);

    loop {
        match update.resume(arg) {
            UpdateCalendarItemResult::Ok => break,
            UpdateCalendarItemResult::Err(err) => panic!("{err}"),
            UpdateCalendarItemResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    }

    let mut arg = None;
    let mut list = ListCalendarItems::new(root, &calendar.id);

    let items = loop {
        match list.resume(arg) {
            ListCalendarItemsResult::Ok(items) => break items,
            ListCalendarItemsResult::Err(err) => panic!("{err}"),
            ListCalendarItemsResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    };

    assert_eq!(items.len(), 1);

    let first_item = items.into_iter().next().unwrap();

    assert_eq!(
        first_item.to_string(),
        "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nDTSTART:19970714T170000Z\r\nSUMMARY:Test2\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    );

    // should delete calendar item

    let mut arg = None;
    let mut delete = DeleteCalendarItem::new(root, &calendar.id, &first_item.id);

    loop {
        match delete.resume(arg) {
            DeleteCalendarItemResult::Ok => break,
            DeleteCalendarItemResult::Err(err) => panic!("{err}"),
            DeleteCalendarItemResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    }

    let mut arg = None;
    let mut list = ListCalendarItems::new(root, &calendar.id);

    let items = loop {
        match list.resume(arg) {
            ListCalendarItemsResult::Ok(items) => break items,
            ListCalendarItemsResult::Err(err) => panic!("{err}"),
            ListCalendarItemsResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    };

    assert_eq!(items.into_iter().count(), 0);

    // should delete calendar

    let mut arg = None;
    let mut delete = DeleteCalendar::new(root, &calendar.id);

    loop {
        match delete.resume(arg) {
            DeleteCalendarResult::Ok => break,
            DeleteCalendarResult::Err(err) => panic!("{err}"),
            DeleteCalendarResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    }

    let mut arg = None;
    let mut list = ListCalendars::new(root);

    let calendars = loop {
        match list.resume(arg) {
            ListCalendarsResult::Ok(calendars) => break calendars,
            ListCalendarsResult::Err(err) => panic!("{err}"),
            ListCalendarsResult::Io(io) => arg = Some(handle(io).unwrap()),
        }
    };

    assert!(calendars.is_empty());
}
