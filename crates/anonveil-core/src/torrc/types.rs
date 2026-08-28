//! Typed input to [`super::generate::build_torrc_fragment`].

/// Configuration for the AnonVeil-managed torrc drop-in fragment
/// (`/etc/tor/torrc.d/anonveil.conf`, written by `anonveil-priv`).
///
/// AnonVeil never edits the system's main `/etc/tor/torrc` — only this
/// fragment, which the distro's `tor` package includes via
/// `%include torrc.d/*.conf` (patched in by AnonVeil's packaging if
/// missing; see `packaging/`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorConfig {
    /// Loopback port Tor's `TransPort` listens on.
    pub trans_port: u16,
    /// Loopback port Tor's `DNSPort` listens on.
    pub dns_port: u16,
    /// Loopback port Tor's `ControlPort` listens on.
    pub control_port: u16,
    /// Tor's data directory (also where the cookie-auth file lives, at
    /// `<data_dir>/control_auth_cookie`, though AnonVeil always reads
    /// the actual path back from `PROTOCOLINFO` rather than assuming it).
    pub data_dir: String,
    /// Whether to use pluggable-transport bridges at all. Off by default —
    /// see `config::schema::BridgeConfig` for the user-facing side of this.
    pub bridges_enabled: bool,
    /// Raw `Bridge` lines (obtained by the user from
    /// bridges.torproject.org or a contact) — passed through verbatim,
    /// never generated or fetched by AnonVeil itself.
    pub bridge_lines: Vec<String>,
    /// Two-letter country codes (or relay fingerprints) exit circuits are
    /// constrained to. Empty means no constraint (Tor's own default
    /// selection).
    pub exit_nodes: Vec<String>,
    /// Country codes (or fingerprints) exit circuits must avoid.
    pub exclude_exit_nodes: Vec<String>,
    /// Whether `exit_nodes`/`exclude_exit_nodes` are a hard requirement
    /// (`StrictNodes 1`) rather than a preference Tor may fall back from
    /// if it can't otherwise build a circuit.
    pub strict_exit_nodes: bool,
}

impl Default for TorConfig {
    fn default() -> Self {
        Self {
            trans_port: 9040,
            dns_port: 5353,
            control_port: 9051,
            data_dir: "/var/lib/tor".to_string(),
            bridges_enabled: false,
            bridge_lines: Vec::new(),
            exit_nodes: Vec::new(),
            exclude_exit_nodes: Vec::new(),
            strict_exit_nodes: false,
        }
    }
}
