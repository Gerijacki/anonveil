//! A minimal, first-party Tor control-port client.
//!
//! `anonveil` deliberately does not depend on an external control-port
//! crate: at the time this was written, the two options on crates.io
//! (`torut`, last published in 2021, and `stem-rs`, brand new and
//! effectively unvetted) were both unsuitable for code sitting on a
//! security-critical, privileged path. The wire protocol itself
//! (control-spec.txt) is a small line-oriented text protocol, so a
//! first-party implementation of just the handful of commands AnonVeil
//! needs (`PROTOCOLINFO`, `AUTHENTICATE`, `SIGNAL NEWNYM`, `GETINFO`) is
//! a few hundred lines, fully unit-tested against a mock transport, and
//! auditable in full by anyone reviewing this project.

pub mod client;
pub mod cookie;
pub mod protocol;

pub use client::{ControlClient, ProtocolInfo};
pub use cookie::encode_cookie_hex;
pub use protocol::Reply;
