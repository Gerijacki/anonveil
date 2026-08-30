//! `anonveil audit-ruleset` — print the *exact* nftables script `start`
//! would load, without touching the system or requiring root. Backs the
//! "audit this yourself" claim in README.md/CONTRIBUTING.md with an
//! actual command instead of just "go read the source".

use anonveil_core::config::AnonveilConfig;
use anonveil_core::firewall::{build_ruleset, render};
use anyhow::Result;

use crate::style;

/// Used only when the real `tor`/`debian-tor` user can't be resolved
/// (most likely `tor` isn't installed yet) — a documented, visibly-
/// flagged stand-in so this command still works as a pre-install preview
/// tool, not a hard requirement to have `tor` set up first. Never used
/// for an actual `start`, which always resolves the real uid.
const PLACEHOLDER_TOR_UID: u32 = 999;

pub fn run(config: &AnonveilConfig) -> Result<()> {
    // Resolving the `tor` user's uid only reads /etc/passwd — no
    // privileged syscall, so this (unlike `start`) never needs root.
    let tor_uid = match anonveil_priv::privilege::resolve_tor_uid() {
        Ok(uid) => uid,
        Err(e) => {
            style::warn(&format!(
                "could not resolve the real `tor` system user ({e}) — showing the ruleset with \
                 a placeholder uid ({PLACEHOLDER_TOR_UID}) instead. Install `tor` and re-run \
                 for the exact ruleset `start` would actually load."
            ));
            PLACEHOLDER_TOR_UID
        }
    };
    let fw_config = config.to_firewall_config(tor_uid);
    let ruleset = build_ruleset(&fw_config);
    print!("{}", render(&ruleset));
    Ok(())
}
