//! Vdir calendar update coroutine wrapping
//! [`io_vdir::collection::update::VdirCollectionUpdate`].
//!
//! The diff merge against the current collection happens in the client
//! method (which lists collections first); this coroutine only writes
//! the already-merged metadata.
//!
//! # Example
//!
//! ```rust,ignore
//! use io_calendar::{
//!     calendar::vdir::update::VdirCalendarUpdate, vdir::client::VdirClient,
//! };
//!
//! let client = VdirClient::new("/path/to/vdir");
//! let coroutine = VdirCalendarUpdate::new(
//!     client.inner.root(),
//!     "personal",
//!     "Personal".into(),
//!     None,
//!     None,
//! );
//! client.run(coroutine)?;
//! ```

use alloc::string::String;

use io_vdir::{
    collection::{
        Collection,
        update::{VdirCollectionUpdate, VdirCollectionUpdateError, VdirCollectionUpdateOptions},
    },
    coroutine::*,
    path::VdirPath,
};
use log::trace;
use thiserror::Error;

use crate::vdir::convert::calendar_path;

/// Errors produced by [`VdirCalendarUpdate`].
#[derive(Debug, Error)]
pub enum VdirCalendarUpdateError {
    #[error(transparent)]
    Update(#[from] VdirCollectionUpdateError),
}

/// I/O-free coroutine rewriting a Vdir calendar's metadata.
pub struct VdirCalendarUpdate {
    inner: VdirCollectionUpdate,
}

impl VdirCalendarUpdate {
    /// Builds the coroutine writing the already-merged metadata of
    /// calendar `id` under `root`.
    pub fn new(
        root: &VdirPath,
        id: &str,
        name: String,
        description: Option<String>,
        color: Option<String>,
    ) -> Self {
        trace!("prepare vdir calendar update");

        let collection = Collection {
            path: calendar_path(root, id),
            display_name: Some(name),
            description,
            color,
        };

        Self {
            inner: VdirCollectionUpdate::new(collection, VdirCollectionUpdateOptions::default()),
        }
    }
}

impl VdirCoroutine for VdirCalendarUpdate {
    type Yield = VdirYield;
    type Return = Result<(), VdirCalendarUpdateError>;

    fn resume(&mut self, arg: Option<VdirReply>) -> VdirCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            VdirCoroutineState::Yielded(y) => VdirCoroutineState::Yielded(y),
            VdirCoroutineState::Complete(r) => VdirCoroutineState::Complete(r.map_err(Into::into)),
        }
    }
}
