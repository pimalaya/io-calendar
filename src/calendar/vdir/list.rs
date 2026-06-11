//! Vdir list-calendars coroutine wrapping
//! [`io_vdir::collection::list::VdirCollectionList`]: enumerates every
//! collection directly under the vdir root and maps each to a shared
//! [`Calendar`].
//!
//! # Example
//!
//! ```rust,ignore
//! use io_calendar::{
//!     calendar::vdir::list::VdirCalendarList, vdir::client::VdirClient,
//! };
//!
//! let client = VdirClient::new("/path/to/vdir");
//! let calendars = client.run(VdirCalendarList::new(client.inner.root().clone()))?;
//! ```

use alloc::vec::Vec;

use io_vdir::{
    collection::list::{VdirCollectionList, VdirCollectionListError, VdirCollectionListOptions},
    coroutine::*,
    path::VdirPath,
};
use log::trace;
use thiserror::Error;

use crate::calendar::Calendar;

/// Errors produced by [`VdirCalendarList`].
#[derive(Debug, Error)]
pub enum VdirCalendarListError {
    #[error(transparent)]
    List(#[from] VdirCollectionListError),
}

/// I/O-free coroutine listing every calendar under a vdir root.
pub struct VdirCalendarList {
    inner: VdirCollectionList,
}

impl VdirCalendarList {
    pub fn new(root: impl Into<VdirPath>) -> Self {
        trace!("prepare vdir calendar listing");
        Self {
            inner: VdirCollectionList::new(root, VdirCollectionListOptions::default()),
        }
    }
}

impl VdirCoroutine for VdirCalendarList {
    type Yield = VdirYield;
    type Return = Result<Vec<Calendar>, VdirCalendarListError>;

    fn resume(&mut self, arg: Option<VdirReply>) -> VdirCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            VdirCoroutineState::Yielded(y) => VdirCoroutineState::Yielded(y),
            VdirCoroutineState::Complete(Ok(collections)) => {
                let mut calendars: Vec<Calendar> =
                    collections.into_iter().map(Calendar::from).collect();
                calendars.sort_by(|a, b| a.name.cmp(&b.name));
                VdirCoroutineState::Complete(Ok(calendars))
            }
            VdirCoroutineState::Complete(Err(err)) => VdirCoroutineState::Complete(Err(err.into())),
        }
    }
}
