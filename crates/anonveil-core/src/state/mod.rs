//! Activation-state snapshot: what `anonveil-priv` needs to remember at
//! `start` in order to cleanly restore the host's pre-activation network
//! configuration at `stop`.
//!
//! Only the data structure and its (de)serialization live here — reading
//! and writing `/var/lib/anonveil/state.json` itself is `anonveil-priv`'s
//! job, since that's a privileged filesystem operation.

use serde::{Deserialize, Serialize};

use crate::error::CoreResult;

/// Where `anonveil-priv` persists the current [`StateSnapshot`] between
/// invocations (e.g. `start` now, `stop` in a later process).
pub const STATE_FILE_PATH: &str = "/var/lib/anonveil/state.json";

/// Directory `anonveil-priv` writes full pre-activation `nft list
/// ruleset` backups into, named by timestamp.
pub const RULESET_BACKUP_DIR: &str = "/var/lib/anonveil/backups";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StateSnapshot {
    /// Whether AnonVeil's protection is currently active.
    pub active: bool,
    /// RFC 3339 timestamp of the most recent `start`. Stored as a plain
    /// string rather than depending on a datetime crate in this
    /// otherwise dependency-light core; `anonveil-priv` is responsible
    /// for formatting it.
    pub activated_at: Option<String>,
    /// Path to the full `nft list ruleset` backup taken immediately
    /// before AnonVeil's table was loaded (everything on the host, not
    /// just AnonVeil's own table — AnonVeil must not assume it's the
    /// only nftables user on the box).
    pub pre_existing_ruleset_backup_path: Option<String>,
    /// Exact prior contents of `/etc/resolv.conf`, captured when it was a
    /// regular file (`None` when it was a symlink — see
    /// `resolv_conf_symlink_target` instead — so `stop` never has to
    /// guess which restoration path applies).
    pub resolv_conf_snapshot: Option<String>,
    /// The exact prior symlink target of `/etc/resolv.conf` (e.g.
    /// `../run/systemd/resolve/stub-resolv.conf`), captured when it was
    /// a symlink rather than a regular file — typically because
    /// `systemd-resolved` or NetworkManager owns it. `stop` recreates
    /// this exact symlink rather than guessing a convention.
    pub resolv_conf_symlink_target: Option<String>,
    /// Informational: whether `systemd-resolved` was detected active
    /// before activation (used for `anonveil status` diagnostics; does
    /// not by itself change how `stop` restores `/etc/resolv.conf` — see
    /// `resolv_conf_symlink_target`).
    pub systemd_resolved_was_active: bool,
    /// Defensive flag: true if `table inet anonveil` was already present
    /// when `start` ran, meaning a previous `stop` didn't clean up
    /// properly. `start` refuses to proceed (rather than double-apply)
    /// when this is true — see `anonveil-priv::apply`.
    pub anonveil_table_pre_existed: bool,
    /// True once `anonveil panic` has additionally loaded the
    /// `inet anonveil_panic` table on top of the state above. Cleared
    /// only by an explicit `anonveil stop --force`.
    pub panic_active: bool,
}

impl StateSnapshot {
    pub fn from_json(contents: &str) -> CoreResult<Self> {
        Ok(serde_json::from_str(contents)?)
    }

    pub fn to_json_pretty(&self) -> CoreResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_json() {
        let snapshot = StateSnapshot {
            active: true,
            activated_at: Some("2026-08-28T12:00:00Z".to_string()),
            pre_existing_ruleset_backup_path: Some(
                "/var/lib/anonveil/backups/ruleset-20260828T120000Z.nft".to_string(),
            ),
            resolv_conf_snapshot: Some("nameserver 1.1.1.1\n".to_string()),
            resolv_conf_symlink_target: None,
            systemd_resolved_was_active: true,
            anonveil_table_pre_existed: false,
            panic_active: false,
        };
        let json = snapshot.to_json_pretty().unwrap();
        let parsed = StateSnapshot::from_json(&json).unwrap();
        assert_eq!(parsed, snapshot);
    }

    #[test]
    fn default_is_inactive() {
        let snapshot = StateSnapshot::default();
        assert!(!snapshot.active);
        assert!(!snapshot.panic_active);
        assert!(snapshot.activated_at.is_none());
    }
}
