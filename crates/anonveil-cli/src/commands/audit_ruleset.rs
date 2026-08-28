//! `anonveil audit-ruleset` — print the *exact* nftables script `start`
//! would load, without touching the system or requiring root. Backs the
//! "audit this yourself" claim in README.md/CONTRIBUTING.md with an
//! actual command instead of just "go read the source".

use anonveil_core::config::AnonveilConfig;
use anonveil_core::firewall::{build_ruleset, render};
use anyhow::{Context, Result};

pub fn run(config: &AnonveilConfig) -> Result<()> {
    // Resolving the `tor` user's uid only reads /etc/passwd — no
    // privileged syscall, so this (unlike `start`) never needs root.
    let tor_uid = anonveil_priv::privilege::resolve_tor_uid()
        .context("could not resolve the `tor` system user")?;
    let fw_config = config.to_firewall_config(tor_uid);
    let ruleset = build_ruleset(&fw_config);
    print!("{}", render(&ruleset));
    Ok(())
}
