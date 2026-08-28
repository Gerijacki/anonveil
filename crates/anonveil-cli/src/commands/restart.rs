//! `anonveil restart` — reapply the kill switch and Tor configuration
//! from scratch (unlike `newnym`, which only requests a new circuit).

use anonveil_core::config::AnonveilConfig;
use anyhow::Result;

use crate::style;

use super::{start, stop};

pub async fn run(config: &AnonveilConfig) -> Result<()> {
    let state = anonveil_priv::snapshot::load_state()?;
    if state.active {
        style::step("stopping current session before restarting...");
        stop::run(false)?;
    }
    start::run(config).await
}
