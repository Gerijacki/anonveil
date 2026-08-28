//! Managing the distro `tor` systemd unit and AnonVeil's torrc drop-in
//! fragment. Never touches `/etc/tor/torrc` itself — only the fragment
//! and (idempotently, once) the `%include` line that pulls it in.

use std::fs;
use std::path::Path;

use anonveil_core::torrc::{build_torrc_fragment, TorConfig};
use tracing::info;

use crate::error::{PrivError, PrivResult};
use crate::exec::run;

const TORRC_PATH: &str = "/etc/tor/torrc";
const TORRC_D_DIR: &str = "/etc/tor/torrc.d";
const TORRC_FRAGMENT_PATH: &str = "/etc/tor/torrc.d/anonveil.conf";
const INCLUDE_LINE: &str = "%include /etc/tor/torrc.d/*.conf";

/// Ensure `/etc/tor/torrc` pulls in `torrc.d/*.conf`. Idempotent: does
/// nothing if an equivalent include is already present. AnonVeil's
/// packaging (`packaging/arch/anonveil.install`, the Debian postinst)
/// already does this at install time — this is the defensive runtime
/// fallback for a manual/`install.sh` install.
pub fn ensure_torrc_include() -> PrivResult<()> {
    let existing = fs::read_to_string(TORRC_PATH).unwrap_or_default();
    if existing.lines().any(|l| l.trim() == INCLUDE_LINE) {
        return Ok(());
    }
    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str("\n# Added by AnonVeil so it can manage its own settings via a drop-in.\n");
    updated.push_str(INCLUDE_LINE);
    updated.push('\n');
    fs::write(TORRC_PATH, updated)?;
    info!("patched /etc/tor/torrc to include torrc.d/*.conf");
    Ok(())
}

/// Write (or overwrite) AnonVeil's torrc drop-in fragment.
pub fn write_torrc_fragment(config: &TorConfig) -> PrivResult<()> {
    fs::create_dir_all(TORRC_D_DIR)?;
    fs::write(TORRC_FRAGMENT_PATH, build_torrc_fragment(config))?;
    Ok(())
}

/// Remove AnonVeil's torrc fragment (on `stop`), returning the daemon to
/// its distro-default configuration.
pub fn remove_torrc_fragment() -> PrivResult<()> {
    let path = Path::new(TORRC_FRAGMENT_PATH);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// The two possible systemd unit names for the Tor daemon across the
/// target distros, tried **in this order** — order matters and is not
/// arbitrary. Debian/Ubuntu's `tor` package supports multiple instances
/// via `tor@<name>.service`, with `tor@default.service` owning
/// `/etc/tor/torrc` and actually running the daemon; on at least some
/// Debian/Ubuntu images `tor.service` is *not* an alias for it but a
/// separate, mostly-inert "multi-instance-master" unit that
/// `reload-or-restart` happily succeeds against without ever touching the
/// real daemon or its config — confirmed the hard way by the
/// `e2e-smoke-test` CI job, which reload-succeeded, then got
/// `Connection refused` trying to reach a control port nothing was
/// actually listening on. Trying the instance-specific name first is safe
/// on Arch too: Arch's `tor` package has no multi-instance support, so
/// `tor@default.service` simply doesn't exist there and this falls
/// through to `tor.service`, which is correct on Arch.
const TOR_UNIT_CANDIDATES: [&str; 2] = ["tor@default.service", "tor.service"];

fn systemctl_reload_or_restart(unit: &str) -> PrivResult<()> {
    run("systemctl", &["reload-or-restart", unit])?;
    Ok(())
}

/// Reload (or restart, if reload isn't supported) the Tor daemon so it
/// picks up AnonVeil's torrc fragment. Tries each known unit name until
/// one succeeds.
pub fn reload_tor() -> PrivResult<()> {
    let mut last_err = None;
    for unit in TOR_UNIT_CANDIDATES {
        match systemctl_reload_or_restart(unit) {
            Ok(()) => {
                info!(unit, "tor service reloaded");
                return Ok(());
            }
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or(PrivError::CommandNotFound("systemctl".to_string())))
}

/// Whether the `tor` package/service appears to be installed at all,
/// checked before `start` does anything else so the error message is
/// actionable instead of a confusing failure three steps later.
pub fn tor_service_installed() -> bool {
    TOR_UNIT_CANDIDATES.iter().any(|unit| {
        run("systemctl", &["list-unit-files", unit, "--no-legend"])
            .is_ok_and(|out| !out.trim().is_empty())
    })
}
