//! `/etc/resolv.conf` snapshot/override/restore.
//!
//! AnonVeil points the system at a plain `nameserver 127.0.0.1` while
//! active — the actual redirection from port 53 to Tor's `DNSPort`
//! happens in the firewall ruleset (see `anonveil_core::firewall::nft`),
//! not here. This module's only job is making sure whatever previously
//! owned `/etc/resolv.conf` (a static file, or a `systemd-resolved`/
//! NetworkManager symlink) comes back exactly as it was on `stop`.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use crate::error::PrivResult;

const RESOLV_CONF: &str = "/etc/resolv.conf";

/// What `/etc/resolv.conf` looked like before AnonVeil touched it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvConfState {
    /// Regular-file content, if it was a regular file.
    pub content: Option<String>,
    /// Symlink target, if it was a symlink (e.g. to
    /// `../run/systemd/resolve/stub-resolv.conf`).
    pub symlink_target: Option<String>,
}

/// Snapshot the current state without modifying anything.
pub fn capture() -> PrivResult<ResolvConfState> {
    let path = Path::new(RESOLV_CONF);
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ResolvConfState {
                content: None,
                symlink_target: None,
            })
        }
        Err(e) => return Err(e.into()),
    };
    if meta.file_type().is_symlink() {
        let target = fs::read_link(path)?;
        Ok(ResolvConfState {
            content: None,
            symlink_target: Some(target.to_string_lossy().to_string()),
        })
    } else {
        let content = fs::read_to_string(path)?;
        Ok(ResolvConfState {
            content: Some(content),
            symlink_target: None,
        })
    }
}

/// Replace `/etc/resolv.conf` with a plain file pointing resolution at
/// Tor's redirected DNS. Removes any existing symlink first — writing
/// through a `systemd-resolved` symlink would edit the wrong file.
pub fn point_to_localhost() -> PrivResult<()> {
    let path = Path::new(RESOLV_CONF);
    if fs::symlink_metadata(path).is_ok() {
        fs::remove_file(path)?;
    }
    fs::write(path, "nameserver 127.0.0.1\n")?;
    Ok(())
}

/// Best-effort probe for whether `systemd-resolved` appears to be the
/// active resolver, for `anonveil status` diagnostics only — restoration
/// itself relies solely on the captured [`ResolvConfState`], not this.
pub fn systemd_resolved_looks_active(state: &ResolvConfState) -> bool {
    state
        .symlink_target
        .as_deref()
        .is_some_and(|t| t.contains("systemd/resolve"))
}

/// Restore `/etc/resolv.conf` to exactly the state [`capture`] recorded.
pub fn restore(state: &ResolvConfState) -> PrivResult<()> {
    let path = Path::new(RESOLV_CONF);
    if fs::symlink_metadata(path).is_ok() {
        fs::remove_file(path)?;
    }
    if let Some(target) = &state.symlink_target {
        symlink(target, path)?;
    } else if let Some(content) = &state.content {
        fs::write(path, content)?;
    }
    // If both are None, resolv.conf genuinely didn't exist before —
    // leave it absent rather than inventing content.
    Ok(())
}
