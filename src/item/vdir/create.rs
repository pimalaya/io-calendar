//! Vdir item create coroutine wrapping
//! [`io_vdir::item::store::VdirItemStore`] with a generated id.
//!
//! # Example
//!
//! ```rust,ignore
//! use io_calendar::{item::vdir::create::VdirCalendarItemCreate, vdir::client::VdirClient};
//!
//! let client = VdirClient::new("/path/to/vdir");
//! let path = client.inner.root().join("personal");
//! let id = client.run(VdirCalendarItemCreate::new(path, ical_bytes)?)?;
//! ```

use alloc::{string::String, vec::Vec};

use io_vdir::{
    coroutine::*,
    item::{
        ItemKind,
        store::{VdirItemStore, VdirItemStoreError, VdirItemStoreOptions},
    },
    path::VdirPath,
};
use log::trace;
use thiserror::Error;

/// Errors produced by [`VdirCalendarItemCreate`].
#[derive(Debug, Error)]
pub enum VdirCalendarItemCreateError {
    #[error(transparent)]
    Store(#[from] VdirItemStoreError),
    #[error("Empty item body")]
    EmptyBody,
}

/// I/O-free coroutine writing a new iCalendar item under a collection.
///
/// The id is minted by the inner store coroutine; on completion the
/// generated item id is returned.
pub struct VdirCalendarItemCreate {
    inner: VdirItemStore,
}

impl VdirCalendarItemCreate {
    /// Builds the coroutine storing `contents` as a fresh iCalendar item
    /// under the calendar at `path`, rejecting empty contents.
    pub fn new(
        path: impl Into<VdirPath>,
        contents: Vec<u8>,
    ) -> Result<Self, VdirCalendarItemCreateError> {
        trace!("prepare vdir item create");

        if contents.is_empty() {
            return Err(VdirCalendarItemCreateError::EmptyBody);
        }

        Ok(Self {
            inner: VdirItemStore::new(
                path,
                None,
                ItemKind::Ical,
                contents,
                VdirItemStoreOptions::default(),
            ),
        })
    }
}

impl VdirCoroutine for VdirCalendarItemCreate {
    type Yield = VdirYield;
    type Return = Result<String, VdirCalendarItemCreateError>;

    fn resume(&mut self, arg: Option<VdirReply>) -> VdirCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            VdirCoroutineState::Yielded(y) => VdirCoroutineState::Yielded(y),
            VdirCoroutineState::Complete(Ok(out)) => VdirCoroutineState::Complete(Ok(out.id)),
            VdirCoroutineState::Complete(Err(err)) => VdirCoroutineState::Complete(Err(err.into())),
        }
    }
}
