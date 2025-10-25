#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(feature = "caldav")]
pub mod caldav;
pub mod calendar;
pub mod item;
#[cfg(feature = "vdir")]
pub mod vdir;
