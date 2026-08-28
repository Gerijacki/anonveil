//! Typed inputs and the typed nftables ruleset AST produced by [`super::nft`].

use std::fmt;

/// How AnonVeil's kill switch treats IPv6 traffic.
///
/// v0.1 only implements [`Ipv6Mode::Block`]. `RouteThroughTor` is reserved
/// for a future release (see ROADMAP.md) once Tor's own IPv6 transparent
/// proxying support is wired up end-to-end; selecting it today still
/// results in a hard block, with a warning logged by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ipv6Mode {
    #[default]
    Block,
    RouteThroughTor,
}

/// One extra `accept` carve-out for the kill switch, e.g. so an admin's
/// existing SSH session or a LAN sync port keeps working while AnonVeil
/// is active. Sourced from `config.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcludedTcpPort(pub u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcludedInterface(pub String);

/// Pure input to [`super::nft::build_ruleset`]. Every field that depends
/// on live system state (the `tor` user's uid, chosen ports, user
/// overrides) is passed in explicitly — this struct is the *only* input
/// the rule-generation function needs, which is what keeps it a pure,
/// deterministic function safe to golden-file test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirewallConfig {
    /// Tor's `TransPort` (transparent TCP redirection target).
    pub trans_port: u16,
    /// Tor's `DNSPort` (DNS redirection target).
    pub dns_port: u16,
    /// uid of the `tor` system user, so its own outbound connections are
    /// never redirected back into itself (classic transparent-proxy loop).
    pub tor_uid: u32,
    /// Extra TCP ports on the host itself that bypass the kill switch.
    pub excluded_tcp_ports: Vec<ExcludedTcpPort>,
    /// Interfaces the kill switch never restricts.
    pub excluded_interfaces: Vec<ExcludedInterface>,
    /// IPv6 handling strategy.
    pub ipv6_mode: Ipv6Mode,
}

impl Default for FirewallConfig {
    fn default() -> Self {
        Self {
            trans_port: 9040,
            dns_port: 5353,
            tor_uid: 0, // caller MUST override; 0 is an intentionally invalid sentinel
            excluded_tcp_ports: Vec::new(),
            excluded_interfaces: Vec::new(),
            ipv6_mode: Ipv6Mode::default(),
        }
    }
}

impl FirewallConfig {
    /// The name of AnonVeil's dedicated nftables table. Kept as an
    /// isolated `inet` table (covers IPv4 + IPv6 together) so activation
    /// and teardown never touch any pre-existing rules on the host.
    pub const TABLE_NAME: &'static str = "anonveil";

    /// The RFC1918 + loopback + link-local IPv4 ranges that are always
    /// treated as "local" and exempted from Tor redirection.
    pub fn default_local_nets_v4() -> Vec<&'static str> {
        vec![
            "127.0.0.0/8",
            "10.0.0.0/8",
            "172.16.0.0/12",
            "192.168.0.0/16",
            "169.254.0.0/16",
        ]
    }

    /// Loopback + link-local + unique-local IPv6 ranges, always local.
    pub fn default_local_nets_v6() -> Vec<&'static str> {
        vec!["::1/128", "fe80::/10", "fc00::/7"]
    }
}

/// A single element of an nftables named set (e.g. one CIDR literal).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NftSetElement(pub String);

/// An nftables named set (`local_nets`, `local_nets6`, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NftSet {
    pub name: String,
    /// nftables set element type, e.g. `ipv4_addr` / `ipv6_addr`.
    pub set_type: &'static str,
    /// Whether the set holds intervals/prefixes (`flags interval`).
    pub interval: bool,
    pub elements: Vec<NftSetElement>,
}

/// The nftables hook a base chain attaches to, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NftHook {
    /// `nat` or `filter`.
    pub chain_type: &'static str,
    /// `prerouting`, `output`, `input`, `forward`.
    pub hook: &'static str,
    /// e.g. `dstnat`, `-100`, `filter`.
    pub priority: &'static str,
    /// `accept` or `drop`.
    pub policy: &'static str,
}

/// One nftables chain. `rules` holds fully-formed rule statements in
/// evaluation order — each produced by a small typed builder function in
/// [`super::nft`] rather than hand-assembled strings scattered elsewhere,
/// which is what keeps every individual line traceable to the specific
/// bit of the design it implements and covered by a golden-file test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NftChain {
    pub name: String,
    pub hook: Option<NftHook>,
    pub rules: Vec<String>,
}

/// The full ruleset AST for AnonVeil's dedicated table, ready to be
/// rendered to `nft -f` script text by [`super::render::render`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NftRuleset {
    /// Always `inet` — covers both IPv4 and IPv6 in one table.
    pub table_family: &'static str,
    pub table_name: String,
    pub sets: Vec<NftSet>,
    pub chains: Vec<NftChain>,
}

impl fmt::Display for NftRuleset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&super::render::render(self))
    }
}
