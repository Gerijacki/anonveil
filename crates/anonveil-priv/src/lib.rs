//! `anonveil-priv` — the privileged execution layer.
//!
//! This is the only AnonVeil crate that touches the live system: it
//! shells out to `nft`/`systemctl`/`ip`, reads and writes files under
//! `/etc`, `/run`, and `/var/lib/anonveil`, and opens the real Tor
//! control-port socket. Every rule/config/state value it applies was
//! computed by the pure functions in `anonveil-core`; this crate only
//! ever executes what core already decided.
//!
//! Requires Linux (nftables, systemd, `/etc/tor`, `ip link`) — there is
//! no cross-platform fallback, by design (see `anonveil-core` for the
//! portable logic and its test suite).

pub mod apply;
pub mod control_session;
pub mod error;
pub mod exec;
pub mod mac;
pub mod privilege;
pub mod resolvconf;
pub mod snapshot;
pub mod systemd;

pub use error::{PrivError, PrivResult};
