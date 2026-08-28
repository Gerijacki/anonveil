//! `anonveil start` — activate the kill switch.

use std::time::Duration;

use anonveil_core::config::AnonveilConfig;
use anyhow::{bail, Result};

use crate::style;

/// Best-effort rollback used when activation fails partway through: undo
/// whatever mutating steps already succeeded so the host isn't left in a
/// half-configured state (DNS pointed at Tor with no kill switch loaded,
/// or vice versa). Restoring `/etc/resolv.conf` itself is handled by the
/// caller, which has the pre-activation snapshot in hand.
fn rollback(ruleset_loaded: bool, torrc_written: bool) {
    if ruleset_loaded {
        let _ = anonveil_priv::apply::teardown_main_ruleset();
    }
    if torrc_written {
        let _ = anonveil_priv::systemd::remove_torrc_fragment();
        let _ = anonveil_priv::systemd::reload_tor();
    }
}

pub async fn run(config: &AnonveilConfig) -> Result<()> {
    anonveil_priv::privilege::require_root()?;

    if !anonveil_priv::systemd::tor_service_installed() {
        bail!(
            "the `tor` package doesn't appear to be installed.\n  \
             Arch:   sudo pacman -S tor\n  \
             Debian: sudo apt install tor"
        );
    }

    let mut state = anonveil_priv::snapshot::load_state()?;
    if state.active {
        style::warn("AnonVeil is already active.");
        return Ok(());
    }

    let tor_uid = anonveil_priv::privilege::resolve_tor_uid()?;
    let fw_config = config.to_firewall_config(tor_uid);
    let tor_config = config.to_tor_config();

    style::step("capturing pre-activation state...");
    let mut snapshot = anonveil_priv::snapshot::capture_pre_activation_state()?;
    if snapshot.anonveil_table_pre_existed {
        bail!(
            "AnonVeil's nftables table already exists from a previous session that didn't \
             clean up. Run `anonveil stop --force` first, or `anonveil panic` right now if \
             something looks wrong."
        );
    }

    let mut torrc_written = false;
    let mut dns_pointed = false;
    let mut ruleset_loaded = false;

    let result: Result<()> = async {
        if config.mac.randomize_on_start {
            style::step("randomizing MAC address...");
            match anonveil_priv::mac::default_interface()
                .and_then(|iface| anonveil_priv::mac::randomize(&iface).map(|mac| (iface, mac)))
            {
                Ok((iface, mac)) => style::dim(&format!("  {iface}: now {mac}")),
                // Non-fatal: MAC randomization failing shouldn't block
                // getting the kill switch itself up.
                Err(e) => style::warn(&format!("could not randomize MAC address: {e}")),
            }
        }

        style::step("writing torrc fragment and reloading tor...");
        anonveil_priv::systemd::ensure_torrc_include()?;
        anonveil_priv::systemd::write_torrc_fragment(&tor_config)?;
        anonveil_priv::systemd::reload_tor()?;
        torrc_written = true;

        style::step("pointing DNS at Tor...");
        anonveil_priv::resolvconf::point_to_localhost()?;
        dns_pointed = true;

        style::step("loading kill switch...");
        anonveil_priv::apply::apply_main_ruleset(&fw_config)?;
        ruleset_loaded = true;

        style::step("waiting for tor to finish bootstrapping (this can take a moment)...");
        let mut client =
            anonveil_priv::control_session::connect_and_authenticate(tor_config.control_port)
                .await?;
        anonveil_priv::control_session::wait_for_bootstrap(&mut client, Duration::from_secs(90))
            .await?;

        Ok(())
    }
    .await;

    if let Err(e) = result {
        style::error(&format!("activation failed: {e}"));
        style::step("rolling back partial changes...");
        rollback(ruleset_loaded, torrc_written);
        if dns_pointed {
            // Restore whatever /etc/resolv.conf looked like before this
            // run started, using the snapshot already captured above.
            let resolv_state = anonveil_priv::resolvconf::ResolvConfState {
                content: snapshot.resolv_conf_snapshot.clone(),
                symlink_target: snapshot.resolv_conf_symlink_target.clone(),
            };
            let _ = anonveil_priv::resolvconf::restore(&resolv_state);
        }
        return Err(e);
    }

    snapshot.active = true;
    state = snapshot;
    anonveil_priv::snapshot::save_state(&state)?;

    style::ok("AnonVeil is active — all traffic is now routed through Tor.");
    style::dim("  Run `anonveil status` or `anonveil check` to verify.");
    Ok(())
}
