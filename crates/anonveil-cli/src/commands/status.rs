//! `anonveil status` — a snapshot of current protection state.

use anonveil_core::config::AnonveilConfig;
use anyhow::Result;

use crate::style;

pub async fn run(config: &AnonveilConfig) -> Result<()> {
    let state = anonveil_priv::snapshot::load_state()?;

    if state.panic_active {
        style::warn("PANIC mode is engaged — all traffic is blocked except loopback.");
    }

    if !state.active {
        style::dim("AnonVeil: inactive");
        return Ok(());
    }

    style::ok("AnonVeil: ACTIVE");
    if let Some(since) = &state.activated_at {
        style::dim(&format!("  since: {since}"));
    }

    if !anonveil_priv::snapshot::kill_switch_actually_loaded() {
        style::error(
            "STATE MISMATCH: state.json says active, but the kill switch is NOT actually \
             loaded right now (most likely a reboot dropped it — nftables rules don't survive \
             one on their own). Traffic is currently UNPROTECTED. Run `sudo anonveil start` to \
             reapply it.",
        );
    }

    match anonveil_priv::control_session::connect_and_authenticate(config.network.control_port)
        .await
    {
        Ok(mut client) => {
            match client
                .get_info(&["status/bootstrap-phase", "status/circuit-established"])
                .await
            {
                Ok(info) => {
                    let bootstrapped = info
                        .get("status/bootstrap-phase")
                        .map(|v| v.contains("PROGRESS=100"))
                        .unwrap_or(false);
                    let circuit_established = info
                        .get("status/circuit-established")
                        .map(|v| v == "1")
                        .unwrap_or(false);
                    style::dim(&format!(
                        "  tor bootstrapped: {}",
                        if bootstrapped { "yes" } else { "no" }
                    ));
                    style::dim(&format!(
                        "  circuit established: {}",
                        if circuit_established { "yes" } else { "no" }
                    ));
                }
                Err(e) => style::warn(&format!("could not query tor status: {e}")),
            }
        }
        Err(e) => style::warn(&format!("could not reach tor control port: {e}")),
    }

    style::dim("  run `anonveil check` for a live Tor-reachability self-test");
    Ok(())
}
