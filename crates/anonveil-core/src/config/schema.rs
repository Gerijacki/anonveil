//! `config.toml` schema.
//!
//! Parsing (`toml::from_str`) is pure and lives here; *finding* and
//! *reading* the file (`/etc/anonveil/config.toml`, or `--config`) is
//! `anonveil-cli`'s job.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::CoreResult;
use crate::firewall::{ExcludedInterface, ExcludedTcpPort, FirewallConfig, Ipv6Mode};
use crate::torrc::TorConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Ipv6Setting {
    #[default]
    Block,
    /// Reserved for a future release; currently behaves identically to
    /// `Block` (see `firewall::nft` module docs). Accepted here already
    /// so an existing config doesn't need to change when it lands.
    RouteThroughTor,
}

impl From<Ipv6Setting> for Ipv6Mode {
    fn from(value: Ipv6Setting) -> Self {
        match value {
            Ipv6Setting::Block => Ipv6Mode::Block,
            Ipv6Setting::RouteThroughTor => Ipv6Mode::RouteThroughTor,
        }
    }
}

/// Pluggable-transport bridge configuration (`[network.bridges]`). Off by
/// default — bridges are a real hole-punching-in-a-censored-network
/// feature, not something to turn on implicitly. See `torrc::generate`
/// for exactly what this produces, and `threat-model.md` for what bridges
/// do and don't add over plain Tor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct BridgeConfig {
    pub enabled: bool,
    /// Raw `Bridge` lines from bridges.torproject.org or a contact —
    /// AnonVeil never fetches or invents these itself.
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct NetworkConfig {
    pub trans_port: u16,
    pub dns_port: u16,
    pub control_port: u16,
    pub ipv6_mode: Ipv6Setting,
    pub excluded_tcp_ports: Vec<u16>,
    pub excluded_interfaces: Vec<String>,
    pub bridges: BridgeConfig,
    /// Two-letter country codes (e.g. `"us"`) or relay fingerprints exit
    /// circuits are constrained to. Empty means no constraint. Shrinks
    /// the anonymity set the fewer/more specific entries there are — see
    /// `configuration.md`.
    pub exit_nodes: Vec<String>,
    pub exclude_exit_nodes: Vec<String>,
    /// Whether `exit_nodes`/`exclude_exit_nodes` is a hard requirement
    /// (Tor's `StrictNodes`) rather than a preference.
    pub strict_exit_nodes: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let defaults = TorConfig::default();
        Self {
            trans_port: defaults.trans_port,
            dns_port: defaults.dns_port,
            control_port: defaults.control_port,
            ipv6_mode: Ipv6Setting::default(),
            excluded_tcp_ports: Vec::new(),
            excluded_interfaces: Vec::new(),
            bridges: BridgeConfig::default(),
            exit_nodes: Vec::new(),
            exclude_exit_nodes: Vec::new(),
            strict_exit_nodes: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct MacConfig {
    /// Off by default: MAC randomization changes host behavior in ways
    /// that are surprising if silently enabled. Opt in explicitly.
    pub randomize_on_start: bool,
}

/// One rotation schedule (`[rotation.ip]` or `[rotation.mac]`). Off by
/// default — see `threat-model.md` for why more rotation isn't
/// automatically more private, and for MAC rotation specifically, the
/// real connectivity interruption it causes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RotationScheduleConfig {
    pub enabled: bool,
    pub interval_minutes: u32,
}

/// `[rotation]` — periodic, automatic identity rotation while AnonVeil is
/// active, driven by `anonveil rotate --watch` (see
/// `packaging/systemd/anonveil-rotate.service`, itself opt-in). Distinct
/// from `[mac].randomize_on_start`, which only ever fires once, at
/// `start`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct RotationConfig {
    pub ip: RotationScheduleConfig,
    pub mac: RotationScheduleConfig,
}

impl Default for RotationScheduleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_minutes: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TuiTheme {
    #[default]
    Matrix,
    Cyan,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct TuiConfig {
    pub theme: TuiTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

/// The full parsed contents of `config.toml`. Every section has field
/// defaults, so an empty (or partially specified) file is valid.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct AnonveilConfig {
    pub network: NetworkConfig,
    pub mac: MacConfig,
    pub rotation: RotationConfig,
    pub tui: TuiConfig,
    pub logging: LoggingConfig,
}

impl FromStr for AnonveilConfig {
    type Err = crate::error::CoreError;

    fn from_str(contents: &str) -> CoreResult<Self> {
        Ok(toml::from_str(contents)?)
    }
}

impl AnonveilConfig {
    pub fn to_toml_string(&self) -> CoreResult<String> {
        Ok(toml::to_string_pretty(self)?)
    }

    /// Build the [`FirewallConfig`] this config describes. `tor_uid` must
    /// be resolved by the caller (`anonveil-priv`) — see `firewall::nft`.
    pub fn to_firewall_config(&self, tor_uid: u32) -> FirewallConfig {
        FirewallConfig {
            trans_port: self.network.trans_port,
            dns_port: self.network.dns_port,
            tor_uid,
            excluded_tcp_ports: self
                .network
                .excluded_tcp_ports
                .iter()
                .map(|p| ExcludedTcpPort(*p))
                .collect(),
            excluded_interfaces: self
                .network
                .excluded_interfaces
                .iter()
                .cloned()
                .map(ExcludedInterface)
                .collect(),
            ipv6_mode: self.network.ipv6_mode.clone().into(),
        }
    }

    pub fn to_tor_config(&self) -> TorConfig {
        TorConfig {
            trans_port: self.network.trans_port,
            dns_port: self.network.dns_port,
            control_port: self.network.control_port,
            data_dir: TorConfig::default().data_dir,
            bridges_enabled: self.network.bridges.enabled,
            bridge_lines: self.network.bridges.lines.clone(),
            exit_nodes: self.network.exit_nodes.clone(),
            exclude_exit_nodes: self.network.exclude_exit_nodes.clone(),
            strict_exit_nodes: self.network.strict_exit_nodes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_parses_to_defaults() {
        let config = AnonveilConfig::from_str("").unwrap();
        assert_eq!(config, AnonveilConfig::default());
        assert_eq!(config.network.trans_port, 9040);
        assert_eq!(config.network.dns_port, 5353);
        assert!(!config.mac.randomize_on_start);
    }

    #[test]
    fn round_trips_through_toml() {
        let mut config = AnonveilConfig::default();
        config.network.excluded_tcp_ports.push(22);
        config
            .network
            .excluded_interfaces
            .push("tailscale0".to_string());
        config.mac.randomize_on_start = true;

        let text = config.to_toml_string().unwrap();
        let parsed = AnonveilConfig::from_str(&text).unwrap();
        assert_eq!(parsed, config);
    }

    #[test]
    fn parses_partial_override() {
        let text = r#"
[network]
dns_port = 5454
excluded_tcp_ports = [22, 443]
"#;
        let config = AnonveilConfig::from_str(text).unwrap();
        assert_eq!(config.network.dns_port, 5454);
        assert_eq!(config.network.trans_port, 9040); // untouched default
        assert_eq!(config.network.excluded_tcp_ports, vec![22, 443]);
    }

    #[test]
    fn rotation_off_by_default() {
        let config = AnonveilConfig::default();
        assert!(!config.rotation.ip.enabled);
        assert!(!config.rotation.mac.enabled);
        assert_eq!(config.rotation.ip.interval_minutes, 10);
    }

    #[test]
    fn parses_rotation_section() {
        let text = r#"
[rotation.ip]
enabled = true
interval_minutes = 5

[rotation.mac]
enabled = true
interval_minutes = 45
"#;
        let config = AnonveilConfig::from_str(text).unwrap();
        assert!(config.rotation.ip.enabled);
        assert_eq!(config.rotation.ip.interval_minutes, 5);
        assert!(config.rotation.mac.enabled);
        assert_eq!(config.rotation.mac.interval_minutes, 45);
    }

    #[test]
    fn converts_to_firewall_config() {
        let config = AnonveilConfig::default();
        let fw = config.to_firewall_config(123);
        assert_eq!(fw.tor_uid, 123);
        assert_eq!(fw.trans_port, config.network.trans_port);
        assert_eq!(fw.ipv6_mode, Ipv6Mode::Block);
    }
}
