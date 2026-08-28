//! An exclusive, process-lifetime lock guarding every AnonVeil command
//! that mutates `/var/lib/anonveil/state.json` and the live firewall/DNS/
//! torrc configuration (`start`, `stop`, `panic`; `restart` is just `stop`
//! then `start`, each already locking itself).
//!
//! Without this, two concurrent invocations (e.g. a user running `anonveil
//! start` twice by accident, or a script and a human racing) can both pass
//! their own "is it already active?" check before either has written its
//! result, then both proceed to load/tear down real nftables state and
//! stomp on each other's `state.json` write — the failure mode ranges from
//! a confusing error to a host left in a genuinely inconsistent state
//! (kill switch loaded, DNS not pointed at Tor, or vice versa).

use std::fs::{File, OpenOptions};

use nix::fcntl::{Flock, FlockArg};

use crate::error::{PrivError, PrivResult};

const LOCK_DIR: &str = "/run/anonveil";
const LOCK_PATH: &str = "/run/anonveil/anonveil.lock";

/// Held for as long as the guard is alive; released automatically when it
/// drops (including on an early return via `?`) — [`Flock`]'s own `Drop`
/// impl unlocks the file.
pub struct StateLock(#[allow(dead_code)] Flock<File>);

impl StateLock {
    /// Acquire the lock immediately, failing with
    /// [`PrivError::AnotherOperationInProgress`] rather than blocking if
    /// another AnonVeil command already holds it — a state-mutating CLI
    /// command should tell the user to retry, never hang waiting for one.
    pub fn acquire() -> PrivResult<Self> {
        std::fs::create_dir_all(LOCK_DIR)?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(LOCK_PATH)?;
        let flock = Flock::lock(file, FlockArg::LockExclusiveNonblock)
            .map_err(|(_file, _errno)| PrivError::AnotherOperationInProgress)?;
        Ok(Self(flock))
    }
}
