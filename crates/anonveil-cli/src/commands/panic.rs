//! `anonveil panic` — instant, unconditional kill switch.

use anyhow::Result;

use crate::style;

pub fn run() -> Result<()> {
    anonveil_priv::privilege::require_root()?;

    anonveil_priv::apply::apply_panic()?;

    let mut state = anonveil_priv::snapshot::load_state()?;
    state.panic_active = true;
    anonveil_priv::snapshot::save_state(&state)?;

    style::warn("PANIC: all network traffic has been cut except loopback.");
    style::dim("  Run `anonveil stop --force` once it's safe to restore connectivity.");
    Ok(())
}
