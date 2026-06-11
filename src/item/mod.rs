//! Calendar items: the shared [`CalendarItem`] type plus its
//! per-protocol coroutines (create, delete, get, list, update).

#[cfg(feature = "vdir")]
pub mod vdir;
#[cfg(feature = "webdav")]
pub mod webdav;

mod types;

#[doc(inline)]
pub use types::*;
