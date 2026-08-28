//! `anonveil newnym` — request a fresh Tor circuit/identity without
//! touching the firewall.

use anonveil_core::config::AnonveilConfig;
use anyhow::Result;

use crate::style;

pub async fn run(config: &AnonveilConfig) -> Result<()> {
    let mut client =
        anonveil_priv::control_session::connect_and_authenticate(config.network.control_port)
            .await?;
    client.signal_newnym().await?;
    style::ok("Requested a new Tor circuit/identity.");
    Ok(())
}
