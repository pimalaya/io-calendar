//! WebDAV (CalDAV) coroutines mapping calendar collection operations
//! onto the io-webdav RFC 4791 coroutines.

pub mod create;
pub mod delete;
pub mod list;
pub mod update;
