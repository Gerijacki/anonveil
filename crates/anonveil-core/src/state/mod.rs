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
    /// True only once [`resolv_conf_snapshot`]/[`resolv_conf_symlink_target`]
    /// were actually populated by a real pre-activation capture (i.e. a
    /// genuine `start` ran). Both those fields are `None`/`None` in two
    /// very different situations — "captured, and `/etc/resolv.conf`
    /// genuinely didn't exist" vs. "never captured at all" (no `start`
    /// ever ran, or the state file was missing/reset) — and confusing the
    /// two is a real host-DNS-destroying bug: `stop` must never delete a
    /// live `/etc/resolv.conf` on the strength of a capture that never
    /// happened. `#[serde(default)]` makes this `false` for any state
    /// file written before this field existed, which is the conservative
    /// (skip restoring) side of that mistake.
    ///
    /// [`resolv_conf_snapshot`]: StateSnapshot::resolv_conf_snapshot
    /// [`resolv_conf_symlink_target`]: StateSnapshot::resolv_conf_symlink_target
    #[serde(default)]
    pub dns_snapshot_captured: bool,
    /// RFC 3339 timestamp of the most recent successful `anonveil rotate
    /// --ip` (manual or via the `--watch` daemon). `None` if it's never
    /// happened. Informational only — surfaced by `status`/the TUI.
    #[serde(default)]
    pub last_ip_rotation: Option<String>,
    /// Same as [`last_ip_rotation`], for the most recent successful MAC
    /// rotation.
    ///
    /// [`last_ip_rotation`]: StateSnapshot::last_ip_rotation
    #[serde(default)]
    pub last_mac_rotation: Option<String>,
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
            dns_snapshot_captured: true,
            last_ip_rotation: Some("2026-08-28T12:05:00Z".to_string()),
            last_mac_rotation: None,
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
        assert!(!snapshot.dns_snapshot_captured);
        assert!(snapshot.last_ip_rotation.is_none());
        assert!(snapshot.last_mac_rotation.is_none());
    }

    /// A state file written before `dns_snapshot_captured` existed (i.e.
    /// missing the key entirely) must deserialize with it `false`, not
    /// fail to parse and not silently default to `true` — `stop` reading
    /// an old-format file must land on the safe "don't touch resolv.conf"
    /// side, never the "I know what to restore" side.
    #[test]
    fn old_state_file_without_the_field_defaults_to_not_captured() {
        let old_format_json = r#"{
            "active": true,
            "activated_at": "2026-08-28T12:00:00Z",
            "pre_existing_ruleset_backup_path": null,
            "resolv_conf_snapshot": null,
            "resolv_conf_symlink_target": null,
            "systemd_resolved_was_active": false,
            "anonveil_table_pre_existed": false,
            "panic_active": false
        }"#;
        let parsed = StateSnapshot::from_json(old_format_json).unwrap();
        assert!(!parsed.dns_snapshot_captured);
    }

    /// Same backward-compatibility contract for the rotation-tracking
    /// fields added after `dns_snapshot_captured`: a state file that
    /// predates them must still parse, defaulting both to `None` (nothing
    /// has ever rotated, which is the truthful state for a file that
    /// predates the feature entirely).
    #[test]
    fn old_state_file_without_rotation_fields_defaults_to_none() {
        let old_format_json = r#"{
            "active": true,
            "activated_at": "2026-08-28T12:00:00Z",
            "pre_existing_ruleset_backup_path": null,
            "resolv_conf_snapshot": null,
            "resolv_conf_symlink_target": null,
            "systemd_resolved_was_active": false,
            "anonveil_table_pre_existed": false,
            "panic_active": false,
            "dns_snapshot_captured": true
        }"#;
        let parsed = StateSnapshot::from_json(old_format_json).unwrap();
        assert!(parsed.last_ip_rotation.is_none());
        assert!(parsed.last_mac_rotation.is_none());
    }
}
