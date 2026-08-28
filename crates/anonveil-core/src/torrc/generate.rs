//! Pure torrc drop-in fragment generation.

use super::types::TorConfig;

/// Wrap a plain country code (`us`) in the `{}` syntax `ExitNodes`/
/// `ExcludeExitNodes` expect (`{us}`) — a bare relay fingerprint (40 hex
/// chars, optionally `$`-prefixed) is passed through unchanged, since
/// those never take braces. Keeping this wrapping in `core` rather than
/// asking the user to type `{us}` in `config.toml` is a small but real
/// usability difference: a stray missing brace there would silently be
/// ignored by Tor rather than erroring.
fn format_node_selector(selector: &str) -> String {
    if selector.len() == 2 && selector.chars().all(|c| c.is_ascii_alphabetic()) {
        format!("{{{}}}", selector.to_ascii_lowercase())
    } else {
        selector.to_string()
    }
}

fn format_node_list(selectors: &[String]) -> String {
    selectors
        .iter()
        .map(|s| format_node_selector(s))
        .collect::<Vec<_>>()
        .join(",")
}

/// Build the exact text of AnonVeil's torrc drop-in fragment.
///
/// `AutomapHostsOnResolve`/`AutomapHostsSuffixes` is what makes `.onion`
/// (and `.exit`) addresses resolve transparently once AnonVeil is
/// active, with no extra browser/app configuration — Tor maps them to a
/// virtual address internally instead of ever attempting a real DNS
/// lookup for them.
pub fn build_torrc_fragment(config: &TorConfig) -> String {
    let mut out = format!(
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
    );

    if !config.exit_nodes.is_empty() {
        out.push_str(&format!(
            "ExitNodes {}\n",
            format_node_list(&config.exit_nodes)
        ));
    }
    if !config.exclude_exit_nodes.is_empty() {
        out.push_str(&format!(
            "ExcludeExitNodes {}\n",
            format_node_list(&config.exclude_exit_nodes)
        ));
    }
    if (!config.exit_nodes.is_empty() || !config.exclude_exit_nodes.is_empty())
        && config.strict_exit_nodes
    {
        out.push_str("StrictNodes 1\n");
    }

    if config.bridges_enabled && !config.bridge_lines.is_empty() {
        out.push_str("UseBridges 1\n");
        out.push_str("ClientTransportPlugin obfs4 exec /usr/bin/obfs4proxy\n");
        for line in &config.bridge_lines {
            out.push_str(&format!("Bridge {line}\n"));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_config_has_no_exit_node_or_bridge_lines() {
        let fragment = build_torrc_fragment(&TorConfig::default());
        assert!(!fragment.contains("ExitNodes"));
        assert!(!fragment.contains("ExcludeExitNodes"));
        assert!(!fragment.contains("StrictNodes"));
        assert!(!fragment.contains("UseBridges"));
        assert!(!fragment.contains("Bridge "));
        assert!(!fragment.contains("ClientTransportPlugin"));
    }

    #[test]
    fn exit_nodes_are_wrapped_in_braces_country_codes_only() {
        let config = TorConfig {
            exit_nodes: vec!["us".to_string(), "de".to_string()],
            exclude_exit_nodes: vec![
                "ru".to_string(),
                "$AAAABBBBCCCCDDDDEEEEFFFF0000111122223333".to_string(),
            ],
            ..TorConfig::default()
        };
        let fragment = build_torrc_fragment(&config);
        assert!(fragment.contains("ExitNodes {us},{de}\n"));
        assert!(
            fragment.contains("ExcludeExitNodes {ru},$AAAABBBBCCCCDDDDEEEEFFFF0000111122223333\n")
        );
    }

    #[test]
    fn strict_nodes_only_emitted_when_a_selector_is_actually_set() {
        let mut config = TorConfig {
            strict_exit_nodes: true,
            ..TorConfig::default()
        };
        // No exit_nodes/exclude_exit_nodes set — StrictNodes on its own
        // means nothing to Tor and would just be noise.
        assert!(!build_torrc_fragment(&config).contains("StrictNodes"));

        config.exit_nodes = vec!["us".to_string()];
        assert!(build_torrc_fragment(&config).contains("StrictNodes 1\n"));
    }

    #[test]
    fn bridges_disabled_by_default_even_with_lines_configured() {
        let config = TorConfig {
            bridge_lines: vec!["obfs4 1.2.3.4:443 CERT=abc".to_string()],
            ..TorConfig::default()
        };
        // bridges_enabled stays false — lines alone must not activate it.
        assert!(!build_torrc_fragment(&config).contains("UseBridges"));
    }

    #[test]
    fn enabled_bridges_emit_the_transport_plugin_and_every_line() {
        let config = TorConfig {
            bridges_enabled: true,
            bridge_lines: vec![
                "obfs4 1.2.3.4:443 CERT=abc IAT-MODE=0".to_string(),
                "obfs4 5.6.7.8:443 CERT=def IAT-MODE=0".to_string(),
            ],
            ..TorConfig::default()
        };
        let fragment = build_torrc_fragment(&config);
        assert!(fragment.contains("UseBridges 1\n"));
        assert!(fragment.contains("ClientTransportPlugin obfs4 exec /usr/bin/obfs4proxy\n"));
        assert!(fragment.contains("Bridge obfs4 1.2.3.4:443 CERT=abc IAT-MODE=0\n"));
        assert!(fragment.contains("Bridge obfs4 5.6.7.8:443 CERT=def IAT-MODE=0\n"));
    }
}
