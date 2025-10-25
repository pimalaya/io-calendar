use std::{collections::HashSet, path::Path};

use io_fs::io::FsIo;
use io_vdir::coroutines::list_collections::{
    ListCollections, ListCollectionsError, ListCollectionsResult,
};
use thiserror::Error;

use crate::calendar::Calendar;

#[derive(Clone, Debug, Error)]
pub enum ListCalendarsError {
    #[error("List calendars error")]
    ListCollections(#[from] ListCollectionsError),
}

#[derive(Clone, Debug)]
pub enum ListCalendarsResult {
    Ok(HashSet<Calendar>),
    Err(ListCalendarsError),
    Io(FsIo),
}

#[derive(Debug)]
pub struct ListCalendars(ListCollections);

impl ListCalendars {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self(ListCollections::new(root))
    }

    pub fn resume(&mut self, input: Option<FsIo>) -> ListCalendarsResult {
        let collections = loop {
            match self.0.resume(input) {
                ListCollectionsResult::Ok(collections) => break collections,
                ListCollectionsResult::Err(err) => return ListCalendarsResult::Err(err.into()),
                ListCollectionsResult::Io(io) => return ListCalendarsResult::Io(io),
            }
        };

        let mut calendars = HashSet::new();

        for collection in collections {
            let Some(id) = collection.path.file_stem() else {
                continue;
            };

            calendars.insert(Calendar {
                id: id.to_string_lossy().to_string(),
                display_name: collection.display_name,
                description: collection.description,
                color: collection.color,
            });
        }

        ListCalendarsResult::Ok(calendars)
    }
}
