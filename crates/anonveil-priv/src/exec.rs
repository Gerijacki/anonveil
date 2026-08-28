//! Thin, logged wrappers around the external commands AnonVeil shells
//! out to (`nft`, `systemctl`, `getent`). Every privileged mutation of
//! the host ultimately funnels through here, which keeps the actual
//! `Command` invocations in one auditable place.

use std::process::Command;

use tracing::debug;

use crate::error::{PrivError, PrivResult};

/// Run `command` with `args`, returning captured stdout as a `String` on
/// success (exit code 0). On failure, returns a [`PrivError`] carrying
/// the exit status and captured stderr — never silently swallowed.
pub fn run(command: &str, args: &[&str]) -> PrivResult<String> {
    debug!(command, ?args, "executing");
    let output = Command::new(command).args(args).output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            PrivError::CommandNotFound(command.to_string())
        } else {
            PrivError::Io(e)
        }
    })?;

    if !output.status.success() {
        return Err(PrivError::CommandFailed {
            command: format!("{command} {}", args.join(" ")),
            status: output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "signal".to_string()),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Like [`run`], but success/failure is reduced to a bool and stderr is
/// discarded — for existence/status checks where a non-zero exit is an
/// expected, non-exceptional outcome (e.g. "does this nftables table
/// exist yet?").
pub fn run_ok(command: &str, args: &[&str]) -> bool {
    debug!(command, ?args, "executing (status-only)");
    Command::new(command)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
