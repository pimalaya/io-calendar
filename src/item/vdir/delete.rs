//! Vdir item delete coroutine wrapping
//! [`io_vdir::item::delete::VdirItemDelete`].
//!
//! # Example
//!
//! ```rust,ignore
//! use io_calendar::{item::vdir::delete::VdirCalendarItemDelete, vdir::client::VdirClient};
//!
//! let client = VdirClient::new("/path/to/vdir");
//! let path = client.inner.root().join("personal");
//! client.run(VdirCalendarItemDelete::new(path, "event-1"))?;
//! ```

use alloc::string::ToString;

use io_vdir::{
    coroutine::*,
    item::delete::{VdirItemDelete, VdirItemDeleteError, VdirItemDeleteOptions},
    path::VdirPath,
};
use log::trace;
use thiserror::Error;

/// Errors produced by [`VdirCalendarItemDelete`].
#[derive(Debug, Error)]
pub enum VdirCalendarItemDeleteError {
    #[error(transparent)]
    Delete(#[from] VdirItemDeleteError),
}

/// I/O-free coroutine locating then removing a Vdir item by its id.
pub struct VdirCalendarItemDelete {
    inner: VdirItemDelete,
}

impl VdirCalendarItemDelete {
    /// Builds the coroutine deleting item `item_id` from the calendar at
    /// `path`.
    pub fn new(path: impl Into<VdirPath>, item_id: impl ToString) -> Self {
        trace!("prepare vdir item delete");
        Self {
            inner: VdirItemDelete::new(path, item_id, VdirItemDeleteOptions::default()),
        }
    }
}

impl VdirCoroutine for VdirCalendarItemDelete {
    type Yield = VdirYield;
    type Return = Result<(), VdirCalendarItemDeleteError>;

    fn resume(&mut self, arg: Option<VdirReply>) -> VdirCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            VdirCoroutineState::Yielded(y) => VdirCoroutineState::Yielded(y),
            VdirCoroutineState::Complete(r) => VdirCoroutineState::Complete(r.map_err(Into::into)),
        }
    }
}
