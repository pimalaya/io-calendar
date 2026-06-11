//! Vdir calendar create coroutine wrapping
//! [`io_vdir::collection::create::VdirCollectionCreate`].
//!
//! # Example
//!
//! ```rust,ignore
//! use io_calendar::{
//!     calendar::vdir::create::VdirCalendarCreate, vdir::client::VdirClient,
//! };
//!
//! let client = VdirClient::new("/path/to/vdir");
//! let coroutine =
//!     VdirCalendarCreate::new(client.inner.root(), "personal", "Personal", None, None)?;
//! client.run(coroutine)?;
//! ```

use alloc::string::ToString;

use io_vdir::{
    collection::{
        Collection,
        create::{VdirCollectionCreate, VdirCollectionCreateError, VdirCollectionCreateOptions},
    },
    coroutine::*,
    path::VdirPath,
};
use log::trace;
use thiserror::Error;

use crate::vdir::convert::calendar_path;

/// Errors produced by [`VdirCalendarCreate`].
#[derive(Debug, Error)]
pub enum VdirCalendarCreateError {
    #[error(transparent)]
    Create(#[from] VdirCollectionCreateError),
    #[error("Invalid calendar id")]
    InvalidId,
}

/// I/O-free coroutine creating a Vdir calendar collection.
pub struct VdirCalendarCreate {
    inner: VdirCollectionCreate,
}

impl VdirCalendarCreate {
    /// Builds the coroutine creating calendar `id` (display name `name`)
    /// under `root`, rejecting an empty id.
    pub fn new(
        root: &VdirPath,
        id: &str,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<Self, VdirCalendarCreateError> {
        trace!("prepare vdir calendar create");

        let trimmed = id.trim_matches('/');
        if trimmed.is_empty() {
            return Err(VdirCalendarCreateError::InvalidId);
        }

        let collection = Collection {
            path: calendar_path(root, trimmed),
            display_name: Some(name.to_string()),
            description: description.map(str::to_string),
            color: color.map(str::to_string),
        };

        Ok(Self {
            inner: VdirCollectionCreate::new(collection, VdirCollectionCreateOptions::default()),
        })
    }
}

impl VdirCoroutine for VdirCalendarCreate {
    type Yield = VdirYield;
    type Return = Result<(), VdirCalendarCreateError>;

    fn resume(&mut self, arg: Option<VdirReply>) -> VdirCoroutineState<Self::Yield, Self::Return> {
        match self.inner.resume(arg) {
            VdirCoroutineState::Yielded(y) => VdirCoroutineState::Yielded(y),
            VdirCoroutineState::Complete(r) => VdirCoroutineState::Complete(r.map_err(Into::into)),
        }
    }
}
