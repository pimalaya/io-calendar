//! Vdir item update coroutine wrapping
//! [`io_vdir::item::store::VdirItemStore`] in overwrite mode.
//!
//! # Example
//!
//! ```rust,ignore
//! use io_calendar::{item::vdir::update::VdirCalendarItemUpdate, vdir::client::VdirClient};
//!
//! let client = VdirClient::new("/path/to/vdir");
//! let path = client.inner.root().join("personal");
//! client.run(VdirCalendarItemUpdate::new(path, "event-1", ical_bytes)?)?;
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

/// Errors produced by [`VdirCalendarItemUpdate`].
#[derive(Debug, Error)]
pub enum VdirCalendarItemUpdateError {
    #[error(transparent)]
    Store(#[from] VdirItemStoreError),
    #[error("Empty item body")]
    EmptyBody,
}

/// I/O-free coroutine overwriting an existing Vdir item's contents.
pub struct VdirCalendarItemUpdate {
    inner: VdirItemStore,
}

impl VdirCalendarItemUpdate {
    /// Builds the coroutine overwriting item `item_id` under the
    /// calendar at `path`, rejecting empty contents.
    pub fn new(
        path: impl Into<VdirPath>,
        item_id: &str,
        contents: Vec<u8>,
    ) -> Result<Self, VdirCalendarItemUpdateError> {
        trace!("prepare vdir item update");

        if contents.is_empty() {
            return Err(VdirCalendarItemUpdateError::EmptyBody);
        }

        let id: String = item_id.into();
        Ok(Self {
            inner: VdirItemStore::new(
                path,
                Some(id),
                ItemKind::Ical,
                contents,
                VdirItemStoreOptions::default(),
            ),
        })
    }
}

impl VdirCoroutine for VdirCalendarItemUpdate {
    type Yield = VdirYield;
    type Return = Result<(), VdirCalendarItemUpdateError>;

    fn resume(&mut self, arg: Option<VdirReply>) -> VdirCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            VdirCoroutineState::Yielded(y) => VdirCoroutineState::Yielded(y),
            VdirCoroutineState::Complete(Ok(_)) => VdirCoroutineState::Complete(Ok(())),
            VdirCoroutineState::Complete(Err(err)) => VdirCoroutineState::Complete(Err(err.into())),
        }
    }
}
