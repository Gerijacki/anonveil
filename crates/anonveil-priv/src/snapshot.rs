//! Persisting and loading [`StateSnapshot`] and the full pre-activation
//! `nft list ruleset` backup it points to.

use std::fs;
use std::path::Path;

use anonveil_core::state::{StateSnapshot, RULESET_BACKUP_DIR, STATE_FILE_PATH};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tracing::info;

use crate::apply::table_exists;
use crate::error::PrivResult;
use crate::exec::run;
use crate::resolvconf;

/// Whether the *live* kill switch (`table inet anonveil`) is actually
/// loaded right now — independent of what `state.json` says.
///
/// nftables rules do not survive a reboot unless something reapplies them
/// (AnonVeil's optional boot unit does; a bare reboot with it disabled
/// does not). `state.active == true` on its own is therefore not proof of
/// protection — callers that report status to a human (`status`, the TUI)
/// must compare it against this, not trust the persisted flag alone.
pub fn kill_switch_actually_loaded() -> bool {
    table_exists("inet", anonveil_core::firewall::FirewallConfig::TABLE_NAME)
}

pub fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Load the persisted state, or a default (inactive) snapshot if none
/// exists yet (e.g. first-ever run).
pub fn load_state() -> PrivResult<StateSnapshot> {
    let path = Path::new(STATE_FILE_PATH);
    if !path.exists() {
        return Ok(StateSnapshot::default());
    }
    let contents = fs::read_to_string(path)?;
    Ok(StateSnapshot::from_json(&contents)?)
}

pub fn save_state(state: &StateSnapshot) -> PrivResult<()> {
    if let Some(parent) = Path::new(STATE_FILE_PATH).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(STATE_FILE_PATH, state.to_json_pretty()?)?;
    Ok(())
}

/// `nft list ruleset` of the *entire* host (not just AnonVeil's table),
/// saved to a timestamped file under [`RULESET_BACKUP_DIR`]. AnonVeil
/// must not assume it's the only nftables user on the box, so this
/// backup exists purely as a manual-recovery safety net — `stop` itself
/// only ever removes AnonVeil's own table (see `apply::teardown_main_ruleset`),
/// it never restores from this file automatically.
pub fn backup_existing_ruleset() -> PrivResult<String> {
    fs::create_dir_all(RULESET_BACKUP_DIR)?;
    let ruleset_text = run("nft", &["list", "ruleset"])?;
    let filename = format!("ruleset-{}.nft", now_rfc3339().replace([':', '.'], "-"));
    let path = format!("{RULESET_BACKUP_DIR}/{filename}");
    fs::write(&path, ruleset_text)?;
    info!(path, "backed up pre-activation nftables ruleset");
    Ok(path)
}

/// Capture everything `start` needs to remember before it changes
/// anything: whether AnonVeil's table already (unexpectedly) exists, a
/// full ruleset backup, and the current `/etc/resolv.conf` state.
pub fn capture_pre_activation_state() -> PrivResult<StateSnapshot> {
    let anonveil_table_pre_existed =
        table_exists("inet", anonveil_core::firewall::FirewallConfig::TABLE_NAME);
    let backup_path = backup_existing_ruleset()?;
    let resolv = resolvconf::capture()?;
    let systemd_resolved_was_active = resolvconf::systemd_resolved_looks_active(&resolv);

    Ok(StateSnapshot {
        active: false, // set true by the caller once activation actually succeeds
        activated_at: Some(now_rfc3339()),
        pre_existing_ruleset_backup_path: Some(backup_path),
        resolv_conf_snapshot: resolv.content,
        resolv_conf_symlink_target: resolv.symlink_target,
        systemd_resolved_was_active,
        anonveil_table_pre_existed,
        panic_active: false,
        dns_snapshot_captured: true,
    })
}
