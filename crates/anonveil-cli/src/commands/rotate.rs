//! `anonveil rotate` — periodic identity rotation for stronger privacy
//! than the one-shot `newnym`/`mac randomize` alone: a new Tor circuit
//! (new exit IP) and/or a freshly randomized MAC address, either as a
//! single manual action or as a long-running `--watch` daemon (meant to
//! be managed by the optional `anonveil-rotate.service` — see
//! `packaging/systemd/`).
//!
//! Read `threat-model.md`'s rotation section before enabling this:
//! rotating more often is not automatically more private — `NEWNYM` only
//! affects circuits built *after* the signal, not connections already
//! open, and over-rotating mostly just adds load. `--watch` adds random
//! jitter to the interval specifically so the rotation cadence itself
//! isn't a fixed, fingerprintable pattern, but that's a mitigation, not a
//! reason to rotate aggressively.

use std::time::Duration;

use anonveil_core::config::AnonveilConfig;
use anyhow::{Context, Result};
use rand::Rng;

use crate::{commands, style};

/// One-shot: with `ip`/`mac` both false, rotates whatever `[rotation]` in
/// `config.toml` has enabled; either flag forces that rotation regardless
/// of its `enabled` value (manual on-demand use — same as `newnym`/`mac
/// randomize` still work standalone, unchanged).
pub async fn run(config: &AnonveilConfig, ip: bool, mac: bool) -> Result<()> {
    let (do_ip, do_mac) = if ip || mac {
        (ip, mac)
    } else {
        (config.rotation.ip.enabled, config.rotation.mac.enabled)
    };

    if !do_ip && !do_mac {
        style::warn(
            "nothing to rotate — enable [rotation.ip]/[rotation.mac] in config.toml, or pass \
             --ip/--mac explicitly.",
        );
        return Ok(());
    }

    if do_mac {
        anonveil_priv::privilege::require_root()?;
    }

    if do_ip {
        rotate_ip(config).await?;
    }
    if do_mac {
        rotate_mac().await?;
    }
    Ok(())
}

/// The long-running process `anonveil-rotate.service` manages: independent
/// jittered timers for IP/MAC, only for whichever `[rotation]` has
/// enabled. Never terminates on its own — exits immediately (rather than
/// idling forever doing nothing) if neither is enabled.
pub async fn watch(config: &AnonveilConfig) -> Result<()> {
    if !config.rotation.ip.enabled && !config.rotation.mac.enabled {
        style::warn(
            "neither [rotation.ip] nor [rotation.mac] is enabled in config.toml — nothing to \
             watch, exiting.",
        );
        return Ok(());
    }
    if config.rotation.mac.enabled {
        anonveil_priv::privilege::require_root()?;
    }

    style::step("rotation daemon started (Ctrl+C to stop)");
    if config.rotation.ip.enabled {
        style::dim(&format!(
            "  IP rotation roughly every {} min (jittered)",
            config.rotation.ip.interval_minutes
        ));
    }
    if config.rotation.mac.enabled {
        style::dim(&format!(
            "  MAC rotation roughly every {} min (jittered)",
            config.rotation.mac.interval_minutes
        ));
    }

    let ip_loop = async {
        if config.rotation.ip.enabled {
            loop {
                sleep_with_jitter(config.rotation.ip.interval_minutes).await;
                if let Err(e) = rotate_ip(config).await {
                    style::error(&format!("scheduled IP rotation failed: {e}"));
                }
            }
        } else {
            std::future::pending::<()>().await;
        }
    };

    let mac_loop = async {
        if config.rotation.mac.enabled {
            loop {
                sleep_with_jitter(config.rotation.mac.interval_minutes).await;
                if let Err(e) = rotate_mac().await {
                    style::error(&format!("scheduled MAC rotation failed: {e}"));
                }
            }
        } else {
            std::future::pending::<()>().await;
        }
    };

    tokio::select! {
        () = ip_loop => unreachable!("ip_loop never returns"),
        () = mac_loop => unreachable!("mac_loop never returns"),
    }
}

/// Sleep for `interval_minutes` with roughly ±15% random jitter, so a
/// process watching for it can't set a clock by AnonVeil's rotation
/// cadence. Locked briefly (not held across the sleep) so this never
/// blocks a manual `start`/`stop`/`panic` for the whole interval.
async fn rotate_ip(config: &AnonveilConfig) -> Result<()> {
    let _lock = anonveil_priv::lock::StateLock::acquire()?;
    commands::newnym::run(config).await?;
    let mut state = anonveil_priv::snapshot::load_state().unwrap_or_default();
    state.last_ip_rotation = Some(anonveil_priv::snapshot::now_rfc3339());
    let _ = anonveil_priv::snapshot::save_state(&state);
    Ok(())
}

async fn rotate_mac() -> Result<()> {
    let _lock = anonveil_priv::lock::StateLock::acquire()?;
    let iface = anonveil_priv::mac::default_interface()?;
    // `mac::randomize` always tries to bring the interface back up even if
    // the address change itself failed, but the "up" step can still fail
    // (e.g. a flaky driver) — if it does, the interface may be genuinely
    // stuck down, which is a much bigger deal than a normal rotation
    // error, especially under `--watch` (which logs and keeps looping
    // rather than stopping). Say so explicitly rather than surfacing a
    // bare `ip link` error.
    let new_mac = anonveil_priv::mac::randomize(&iface).with_context(|| {
        format!(
            "MAC rotation on {iface} failed partway through — the interface may now be down; \
             check `ip link show {iface}` and, if needed, `sudo ip link set dev {iface} up`"
        )
    })?;
    style::ok(&format!("{iface}: MAC address rotated to {new_mac}"));

    // Changing a live interface's MAC requires bringing the link down and
    // back up — there's no way to do it without a brief interruption.
    // On a host where NetworkManager/systemd-networkd actively manage
    // this interface, that bounce can make them notice a "new" link and
    // reassert their own control over /etc/resolv.conf. If AnonVeil is
    // currently active, immediately reapply its DNS override and confirm
    // the kill switch is still there rather than silently trusting that
    // nothing else touched it in the meantime — see threat-model.md.
    let state = anonveil_priv::snapshot::load_state().unwrap_or_default();
    if state.active {
        if let Err(e) = anonveil_priv::resolvconf::point_to_localhost() {
            style::warn(&format!(
                "could not re-assert the DNS override after this MAC rotation: {e}"
            ));
        }
        if !anonveil_priv::snapshot::kill_switch_actually_loaded() {
            style::error(
                "the kill switch is NOT loaded after this MAC rotation — traffic may be \
                 unprotected right now. Run `sudo anonveil start` to reapply it.",
            );
        }
    }

    let mut state = anonveil_priv::snapshot::load_state().unwrap_or_default();
    state.last_mac_rotation = Some(anonveil_priv::snapshot::now_rfc3339());
    let _ = anonveil_priv::snapshot::save_state(&state);
    Ok(())
}

/// Pure: the inclusive `[min, max]` second bounds `sleep_with_jitter`
/// picks a random duration from — roughly ±15% of the configured
/// interval, floored at 1 second either way. Split out from the actual
/// sleep so the arithmetic (in particular, not truncating jitter to zero
/// for short intervals — see the comment inline) is unit-testable without
/// mocking randomness or waiting on a real timer.
fn jitter_bounds_secs(interval_minutes: u32) -> (u64, u64) {
    let base_secs = u64::from(interval_minutes).saturating_mul(60).max(1);
    // Multiply before dividing: doing it the other way around
    // (base_secs / 100 * 15) truncates to 0 for any interval under ~100s
    // via integer division, silently disabling jitter for short intervals
    // instead of scaling it down with them.
    let jitter_span = base_secs.saturating_mul(15) / 100; // ~15%
    if jitter_span == 0 {
        (base_secs, base_secs)
    } else {
        (
            base_secs.saturating_sub(jitter_span).max(1),
            base_secs.saturating_add(jitter_span),
        )
    }
}

async fn sleep_with_jitter(interval_minutes: u32) {
    let (min, max) = jitter_bounds_secs(interval_minutes);
    let actual_secs = if min == max {
        min
    } else {
        rand::thread_rng().gen_range(min..=max)
    };
    tokio::time::sleep(Duration::from_secs(actual_secs)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jitter_scales_down_for_short_intervals_instead_of_disappearing() {
        // 2 minutes = 120s; naive (base/100*15) truncates to 0 here,
        // which is exactly the bug this test guards against.
        let (min, max) = jitter_bounds_secs(2);
        assert!(min < max, "a 2-minute interval should still get jitter");
        assert!(max - min <= 40); // roughly ±15% of 120s
    }

    #[test]
    fn jitter_bounds_are_symmetric_around_the_configured_interval() {
        let (min, max) = jitter_bounds_secs(10);
        let base = 600u64;
        assert!(min < base && base < max);
        assert_eq!(base - min, max - base);
    }

    #[test]
    fn zero_or_tiny_interval_never_produces_an_empty_or_reversed_range() {
        for minutes in [0, 1, 2, 5, 60, 10_000] {
            let (min, max) = jitter_bounds_secs(minutes);
            assert!(min >= 1, "sleeping for 0s would busy-loop");
            assert!(min <= max);
        }
    }
}
