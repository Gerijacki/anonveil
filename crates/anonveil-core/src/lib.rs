//! `anonveil-core` — pure, privilege-free logic for AnonVeil.
//!
//! Nothing in this crate touches the filesystem, spawns processes, or
//! requires root. It only *computes* things: nftables rulesets, torrc
//! fragments, the Tor control-port wire protocol, config parsing, and
//! state-snapshot data structures. Every system call that actually
//! mutates the host (`nft -f`, `systemctl`, writing `/etc/resolv.conf`,
//! opening the real control-port socket, ...) lives in `anonveil-priv`.
//!
//! This split is deliberate: it is what makes the security-critical rule
//! generation in [`firewall`] fully unit-testable in ordinary CI, with no
//! containers, no root, and no real network.

pub mod config;
pub mod control;
pub mod error;
pub mod firewall;
pub mod leakcheck;
pub mod state;
pub mod torrc;

pub use error::{CoreError, CoreResult};
