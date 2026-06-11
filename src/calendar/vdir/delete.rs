//! Vdir calendar delete coroutine wrapping
//! [`io_vdir::collection::delete::VdirCollectionDelete`].
//!
//! # Example
//!
//! ```rust,ignore
//! use io_calendar::{
//!     calendar::vdir::delete::VdirCalendarDelete, vdir::client::VdirClient,
//! };
//!
//! let client = VdirClient::new("/path/to/vdir");
//! client.run(VdirCalendarDelete::new(client.inner.root(), "personal"))?;
//! ```

use io_vdir::{
    collection::delete::{
        VdirCollectionDelete, VdirCollectionDeleteError, VdirCollectionDeleteOptions,
    },
    coroutine::*,
    path::VdirPath,
};
use log::trace;
use thiserror::Error;

use crate::vdir::convert::calendar_path;

/// Errors produced by [`VdirCalendarDelete`].
#[derive(Debug, Error)]
pub enum VdirCalendarDeleteError {
    #[error(transparent)]
    Delete(#[from] VdirCollectionDeleteError),
}

/// I/O-free coroutine recursively removing a Vdir calendar collection.
pub struct VdirCalendarDelete {
    inner: VdirCollectionDelete,
}

impl VdirCalendarDelete {
    /// Builds the coroutine removing calendar `id` under `root`.
    pub fn new(root: &VdirPath, id: &str) -> Self {
        trace!("prepare vdir calendar delete");
        Self {
            inner: VdirCollectionDelete::new(
                calendar_path(root, id),
                VdirCollectionDeleteOptions::default(),
            ),
        }
    }
}

impl VdirCoroutine for VdirCalendarDelete {
    type Yield = VdirYield;
    type Return = Result<(), VdirCalendarDeleteError>;

    fn resume(&mut self, arg: Option<VdirReply>) -> VdirCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            VdirCoroutineState::Yielded(y) => VdirCoroutineState::Yielded(y),
            VdirCoroutineState::Complete(r) => VdirCoroutineState::Complete(r.map_err(Into::into)),
        }
    }
}
