//! Root checks and system-user lookups. The only place `anonveil-priv`
//! talks to `libc`/the kernel directly rather than shelling out.

use nix::unistd::{Uid, User};

use crate::error::{PrivError, PrivResult};

/// Every privileged subcommand must call this first.
pub fn require_root() -> PrivResult<()> {
    if Uid::effective().is_root() {
        Ok(())
    } else {
        Err(PrivError::NotRoot)
    }
}

/// Resolve the uid of the system's `tor` user, trying both the name
/// Arch's `tor` package uses (`tor`) and the name Debian/Ubuntu's uses
/// (`debian-tor`), in that order.
pub fn resolve_tor_uid() -> PrivResult<u32> {
    for candidate in ["tor", "debian-tor"] {
        if let Ok(Some(user)) = User::from_name(candidate) {
            return Ok(user.uid.as_raw());
        }
    }
    Err(PrivError::TorUserNotFound)
}
