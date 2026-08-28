//! Applying and tearing down nftables rulesets on the live system.
//!
//! Every rule this module ever loads was produced by
//! `anonveil_core::firewall::{build_ruleset, build_panic_ruleset}` and
//! rendered by `anonveil_core::firewall::render` — this module never
//! constructs or string-formats a rule itself, it only writes the
//! already-rendered text to disk and hands it to `nft`.

use std::fs;
use std::path::Path;

use anonveil_core::firewall::{
    build_panic_ruleset, build_ruleset, render, FirewallConfig, NftRuleset,
};
use tracing::info;

use crate::error::{PrivError, PrivResult};
use crate::exec::{run, run_ok};

const RUNTIME_DIR: &str = "/run/anonveil";
const RULESET_PATH: &str = "/run/anonveil/ruleset.nft";
const PANIC_RULESET_PATH: &str = "/run/anonveil/panic.nft";

const PANIC_TABLE_FAMILY: &str = "inet";
const PANIC_TABLE_NAME: &str = "anonveil_panic";

fn write_script(path: &str, content: &str) -> PrivResult<()> {
    fs::create_dir_all(RUNTIME_DIR)?;
    fs::write(Path::new(path), content)?;
    Ok(())
}

/// `nft -c -f <path>` — validate syntax/semantics without loading it.
fn check_syntax(path: &str) -> PrivResult<()> {
    run("nft", &["-c", "-f", path])?;
    Ok(())
}

/// `nft -f <path>` — actually load the ruleset.
fn load(path: &str) -> PrivResult<()> {
    run("nft", &["-f", path])?;
    Ok(())
}

/// Whether `table <family> <name>` currently exists on the host.
pub fn table_exists(family: &str, name: &str) -> bool {
    run_ok("nft", &["list", "table", family, name])
}

/// Apply AnonVeil's main ruleset for `config`. Fails with
/// [`PrivError::TableAlreadyExists`] if `inet anonveil` is already
/// loaded — callers (the `start` command) must check
/// [`table_exists`]/the persisted state snapshot themselves before
/// calling this, so that error carries better context; this function's
/// own check is a last-resort safety net, not the primary guard.
pub fn apply_main_ruleset(config: &FirewallConfig) -> PrivResult<()> {
    if table_exists("inet", FirewallConfig::TABLE_NAME) {
        return Err(PrivError::TableAlreadyExists);
    }
    let ruleset = build_ruleset(config);
    apply_ruleset(&ruleset, RULESET_PATH)
}

fn apply_ruleset(ruleset: &NftRuleset, path: &str) -> PrivResult<()> {
    let script = render(ruleset);
    write_script(path, &script)?;
    check_syntax(path)?;
    load(path)?;
    info!(table = %ruleset.table_name, "nftables table loaded");
    Ok(())
}

/// `nft delete table inet anonveil` — the entire teardown, made atomic
/// and safe by the ruleset always living in its own dedicated table.
pub fn teardown_main_ruleset() -> PrivResult<()> {
    if table_exists("inet", FirewallConfig::TABLE_NAME) {
        run(
            "nft",
            &["delete", "table", "inet", FirewallConfig::TABLE_NAME],
        )?;
        info!("nftables table removed");
    }
    Ok(())
}

/// Load the minimal drop-everything-but-loopback panic table on top of
/// whatever else is loaded. Deliberately does not touch/replace the main
/// table first — panic must be instant and must not depend on anything
/// else succeeding first.
pub fn apply_panic() -> PrivResult<()> {
    let ruleset = build_panic_ruleset();
    apply_ruleset(&ruleset, PANIC_RULESET_PATH)
}

pub fn panic_active() -> bool {
    table_exists(PANIC_TABLE_FAMILY, PANIC_TABLE_NAME)
}

pub fn teardown_panic() -> PrivResult<()> {
    if panic_active() {
        run(
            "nft",
            &["delete", "table", PANIC_TABLE_FAMILY, PANIC_TABLE_NAME],
        )?;
        info!("panic table removed");
    }
    Ok(())
}
