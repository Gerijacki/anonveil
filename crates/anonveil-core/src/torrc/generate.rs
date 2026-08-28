//! Pure torrc drop-in fragment generation.

use super::types::TorConfig;

/// Build the exact text of AnonVeil's torrc drop-in fragment.
///
/// `AutomapHostsOnResolve`/`AutomapHostsSuffixes` is what makes `.onion`
/// (and `.exit`) addresses resolve transparently once AnonVeil is
/// active, with no extra browser/app configuration — Tor maps them to a
/// virtual address internally instead of ever attempting a real DNS
/// lookup for them.
pub fn build_torrc_fragment(config: &TorConfig) -> String {
    format!(
        "\
## Managed by AnonVeil — do not edit by hand.
## Regenerate via the AnonVeil config, not manually.
TransPort 127.0.0.1:{trans_port}
DNSPort 127.0.0.1:{dns_port}
AutomapHostsOnResolve 1
AutomapHostsSuffixes .onion,.exit
VirtualAddrNetworkIPv4 10.192.0.0/10
ControlPort 127.0.0.1:{control_port}
CookieAuthentication 1
CookieAuthFileGroupReadable 1
DataDirectory {data_dir}
",
        trans_port = config.trans_port,
        dns_port = config.dns_port,
        control_port = config.control_port,
        data_dir = config.data_dir,
    )
}
