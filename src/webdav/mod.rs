//! WebDAV (CalDAV) backend: the std-blocking [`client`] plus the
//! [`convert`] helpers shared by the WebDAV collection and item
//! coroutines.

#[cfg(feature = "client")]
pub mod client;
pub mod convert;
