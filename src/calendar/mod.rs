//! Calendar collections: the shared [`Calendar`] type plus
//! its per-protocol coroutines (create, delete, list, update).

#[cfg(feature = "vdir")]
pub mod vdir;
#[cfg(feature = "webdav")]
pub mod webdav;

mod types;

#[doc(inline)]
pub use types::*;
