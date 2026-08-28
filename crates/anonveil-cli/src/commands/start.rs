//! `anonveil start` — activate the kill switch.

use std::time::Duration;

use anonveil_core::config::AnonveilConfig;
use anonveil_core::state::StateSnapshot;
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

/// `only_if_previously_active`: used by the optional boot-time unit (see
/// `Commands::BootResume` in `main.rs`) so it can call this unconditionally
/// on every boot without ever *activating* AnonVeil for a host that never
/// had it running — it should only ever resume a session that was already
/// active, never start a fresh one on the system's behalf.
pub async fn run(config: &AnonveilConfig, only_if_previously_active: bool) -> Result<()> {
    anonveil_priv::privilege::require_root()?;
    let _lock = anonveil_priv::lock::StateLock::acquire()?;

    let state = anonveil_priv::snapshot::load_state()?;

    if only_if_previously_active && !state.active && !state.panic_active {
        // Boot unit, and there was nothing active before shutdown to
        // resume — quietly do nothing rather than activating AnonVeil for
        // a host that never asked for it to run automatically.
        return Ok(());
    }

    // nftables rules don't survive a reboot; `state.json` does (it's a
    // plain file). A previous panic engaged before a reboot is therefore
    // now silently gone from the kernel even though the user's last
    // explicit intent was "cut everything" — reapply that lockdown rather
    // than quietly falling through to normal Tor-routed operation without
    // the user's say-so. This subsumes the plain "panic is currently
    // loaded" case too (that's just `panic_currently_loaded` already true).
    let panic_currently_loaded = anonveil_priv::apply::panic_active();
    if panic_currently_loaded || state.panic_active {
        if !panic_currently_loaded {
            style::warn(
                "a previous `anonveil panic` was engaged before this reboot dropped the \
                 in-kernel rules — reapplying it now rather than silently resuming normal \
                 operation.",
            );
            anonveil_priv::apply::apply_panic()?;
        }
        bail!(
            "PANIC lockdown is engaged. Run `anonveil stop --force` once it's safe to restore \
             connectivity."
        );
    }

    if !anonveil_priv::systemd::tor_service_installed() {
        bail!(
            "the `tor` package doesn't appear to be installed.\n  \
             Arch:   sudo pacman -S tor\n  \
             Debian: sudo apt install tor"
        );
    }

    let table_loaded = anonveil_priv::snapshot::kill_switch_actually_loaded();
    if state.active && table_loaded {
        style::warn("AnonVeil is already active.");
        return Ok(());
    }

    let tor_uid = anonveil_priv::privilege::resolve_tor_uid()?;
    let fw_config = config.to_firewall_config(tor_uid);
    let tor_config = config.to_tor_config();

    // Two ways to get here: a genuinely fresh activation, or resuming a
    // session that `state.json` says was active but whose kill switch the
    // kernel has since lost (a reboot, most likely). These need different
    // snapshots: a fresh run must capture what /etc/resolv.conf looks like
    // *right now* so `stop` can restore it later; a resume must NOT
    // recapture, because right now resolv.conf already holds AnonVeil's
    // own override (that's a plain file, and unlike nftables rules it
    // *does* survive a reboot) — recapturing here would wrongly enshrine
    // AnonVeil's own "nameserver 127.0.0.1" as the "original" state and
    // restore back to *that* on the next `stop`.
    let is_resume = state.active && !table_loaded;
    let mut snapshot: StateSnapshot = if is_resume {
        style::warn(
            "AnonVeil's state says active, but the kill switch isn't actually loaded (most \
             likely a reboot dropped it) — reapplying using the original session's saved \
             configuration.",
        );
        state
    } else {
        style::step("capturing pre-activation state...");
        let snap = anonveil_priv::snapshot::capture_pre_activation_state()?;
        if snap.anonveil_table_pre_existed {
            bail!(
                "AnonVeil's nftables table already exists from a previous session that didn't \
                 clean up. Run `anonveil stop --force` first, or `anonveil panic` right now if \
                 something looks wrong."
            );
        }
        snap
    };

    let mut torrc_written = false;
    let mut dns_pointed = false;
    let mut ruleset_loaded = false;

    let result: Result<()> = async {
        if !is_resume && config.mac.randomize_on_start {
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

        // Load the kill switch *before* touching torrc or DNS. Its
        // default-deny already exempts tor_uid/loopback/LAN, so loading it
        // first only ever narrows what's reachable — it never depends on
        // Tor already being reconfigured. Doing it first closes what would
        // otherwise be a real leak window: with the old order (torrc +
        // DNS first, kill switch last), a moment existed where
        // /etc/resolv.conf already pointed at Tor's DNSPort but no
        // redirect/default-deny was loaded yet, so any TCP connection
        // opened by any process during that window went out directly,
        // unproxied, with the host's real source IP.
        style::step("loading kill switch...");
        anonveil_priv::apply::apply_main_ruleset(&fw_config)?;
        ruleset_loaded = true;

        style::step("writing torrc fragment and reloading tor...");
        anonveil_priv::systemd::ensure_torrc_include()?;
        anonveil_priv::systemd::write_torrc_fragment(&tor_config)?;
        anonveil_priv::systemd::reload_tor()?;
        torrc_written = true;

        style::step("pointing DNS at Tor...");
        anonveil_priv::resolvconf::point_to_localhost()?;
        dns_pointed = true;

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
        if dns_pointed && !is_resume {
            // Restore whatever /etc/resolv.conf looked like before this
            // run started, using the snapshot already captured above. On
            // a resume, the snapshot is the *original* session's — never
            // "restore" from it mid-resume, since the goal there is
            // getting back to the resumed state, not unwinding to before
            // the whole multi-boot session ever began.
            let resolv_state = anonveil_priv::resolvconf::ResolvConfState {
                content: snapshot.resolv_conf_snapshot.clone(),
                symlink_target: snapshot.resolv_conf_symlink_target.clone(),
            };
            let _ = anonveil_priv::resolvconf::restore(&resolv_state);
        }
        return Err(e);
    }

    snapshot.active = true;
    anonveil_priv::snapshot::save_state(&snapshot)?;

    style::ok("AnonVeil is active — all traffic is now routed through Tor.");
    style::dim("  Run `anonveil status` or `anonveil check` to verify.");
    Ok(())
}
