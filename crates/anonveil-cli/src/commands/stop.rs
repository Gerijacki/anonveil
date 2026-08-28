//! `anonveil stop` — deactivate the kill switch and restore the host's
//! pre-activation network configuration.

use anyhow::Result;

use crate::style;

pub fn run(force: bool) -> Result<()> {
    anonveil_priv::privilege::require_root()?;

    let mut state = anonveil_priv::snapshot::load_state()?;
    if !state.active && !anonveil_priv::apply::table_exists("inet", "anonveil") && !force {
        style::warn("AnonVeil is not active.");
        return Ok(());
    }

    style::step("removing kill switch...");
    anonveil_priv::apply::teardown_main_ruleset()?;

    if state.dns_snapshot_captured {
        style::step("restoring DNS configuration...");
        let resolv_state = anonveil_priv::resolvconf::ResolvConfState {
            content: state.resolv_conf_snapshot.clone(),
            symlink_target: state.resolv_conf_symlink_target.clone(),
        };
        anonveil_priv::resolvconf::restore(&resolv_state)?;
    } else {
        // No genuine `start` ever captured what /etc/resolv.conf looked
        // like (e.g. this session only ever ran `anonveil panic`, or the
        // state file was missing/reset). Touching resolv.conf now would
        // mean guessing, and the only "safe" guess (both fields absent)
        // is indistinguishable from "we captured an empty file" — so
        // leave the live file alone rather than risk deleting a real one.
        style::dim("  DNS configuration was never captured by AnonVeil — leaving it untouched.");
    }

    style::step("removing torrc fragment...");
    anonveil_priv::systemd::remove_torrc_fragment()?;
    anonveil_priv::systemd::reload_tor()?;

    if state.panic_active || anonveil_priv::apply::panic_active() {
        style::step("clearing panic lockdown...");
        anonveil_priv::apply::teardown_panic()?;
    }

    state.active = false;
    state.panic_active = false;
    anonveil_priv::snapshot::save_state(&state)?;

    style::ok("AnonVeil stopped — your original network configuration has been restored.");
    Ok(())
}
