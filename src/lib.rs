//! A libexpect-alike library that allows low- and high-level
//! interactions with a subprocess.
//!
//! The library is built, and optimized, for telnet/ssh connections with
//! network devices. Screen scraping isn't the most fault-tolerant way of
//! interacting with these devices, but damned if it isn't the most mature
//! and battle-tested, and it requires little-to-no infrastructure.
#![warn(missing_docs)]
#![feature(io)]

extern crate regex;
pub mod base;
