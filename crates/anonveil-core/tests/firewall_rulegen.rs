//! Golden-file / structural tests for the security-critical rule
//! generation in `firewall::nft`. These run with no root, no container,
//! and no real `nft` binary — they only assert on the AST and rendered
//! text produced by pure functions.

use anonveil_core::firewall::{
    build_panic_ruleset, build_ruleset, ExcludedInterface, ExcludedTcpPort, FirewallConfig,
    Ipv6Mode,
};

fn default_config() -> FirewallConfig {
    FirewallConfig {
        trans_port: 9040,
        dns_port: 5353,
        tor_uid: 100,
        excluded_tcp_ports: Vec::new(),
        excluded_interfaces: Vec::new(),
        ipv6_mode: Ipv6Mode::Block,
    }
}

const EXPECTED_DEFAULT_SCRIPT: &str = "\
# Managed by AnonVeil — do not edit by hand.
# Regenerate via the AnonVeil config, not manually.
table inet anonveil {
    set local_nets {
        type ipv4_addr
        flags interval
        elements = { 127.0.0.0/8, 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 169.254.0.0/16 }
    }
    set local_nets6 {
        type ipv6_addr
        flags interval
        elements = { ::1/128, fe80::/10, fc00::/7 }
    }

    chain output {
        type nat hook output priority -100; policy accept;
        skuid 100 return
        meta l4proto { tcp, udp } th dport 53 redirect to :5353
        ip daddr @local_nets return
        meta l4proto tcp redirect to :9040
    }

    chain filter_input {
        type filter hook input priority filter; policy drop;
        ct state { established, related } accept
        iif lo accept
        ip saddr @local_nets accept
    }

    chain filter_output {
        type filter hook output priority filter; policy drop;
        ct state { established, related } accept
        oif lo accept
        skuid 100 accept
        ip daddr @local_nets accept
        udp dport 5353 ip daddr 127.0.0.1 accept
        tcp dport 9040 ip daddr 127.0.0.1 accept
    }

    chain filter_forward {
        type filter hook forward priority filter; policy drop;
    }
}
";

#[test]
fn default_config_renders_exact_expected_script() {
    let ruleset = build_ruleset(&default_config());
    let rendered = anonveil_core::firewall::render(&ruleset);
    assert_eq!(rendered, EXPECTED_DEFAULT_SCRIPT);
}

#[test]
fn tor_uid_loop_prevention_rule_is_always_first_in_nat_and_filter_output() {
    let ruleset = build_ruleset(&default_config());
    let nat_output = ruleset
        .chains
        .iter()
        .find(|c| c.name == "output")
        .expect("nat output chain present");
    assert_eq!(nat_output.rules.first().unwrap(), "skuid 100 return");

    let filter_output = ruleset
        .chains
        .iter()
        .find(|c| c.name == "filter_output")
        .expect("filter_output chain present");
    // First two rules are the generic ct-state/loopback accepts; the
    // tor_uid accept must come immediately after those, before any
    // config-driven exclusion or the general local-net accept.
    assert_eq!(filter_output.rules[2], "skuid 100 accept");
}

#[test]
fn excluded_tcp_port_bypasses_nat_redirect_and_kill_switch() {
    let mut config = default_config();
    config.excluded_tcp_ports.push(ExcludedTcpPort(22));

    let ruleset = build_ruleset(&config);

    let nat_output = ruleset.chains.iter().find(|c| c.name == "output").unwrap();
    assert!(nat_output
        .rules
        .contains(&"tcp dport 22 return".to_string()));
    // Must appear before the general DNS/TCP redirect rules.
    let return_pos = nat_output
        .rules
        .iter()
        .position(|r| r == "tcp dport 22 return")
        .unwrap();
    let redirect_pos = nat_output
        .rules
        .iter()
        .position(|r| r.contains("redirect to :9040"))
        .unwrap();
    assert!(return_pos < redirect_pos);

    let filter_output = ruleset
        .chains
        .iter()
        .find(|c| c.name == "filter_output")
        .unwrap();
    assert!(filter_output
        .rules
        .contains(&"tcp dport 22 accept".to_string()));
}

#[test]
fn excluded_interface_only_affects_filter_chains_not_nat_redirect() {
    let mut config = default_config();
    config
        .excluded_interfaces
        .push(ExcludedInterface("tailscale0".to_string()));

    let ruleset = build_ruleset(&config);

    let nat_output = ruleset.chains.iter().find(|c| c.name == "output").unwrap();
    assert!(
        !nat_output.rules.iter().any(|r| r.contains("tailscale0")),
        "excluded interfaces must not appear in the nat/output chain (oif is not reliably \
         matchable there — see firewall::nft module docs)"
    );

    let filter_output = ruleset
        .chains
        .iter()
        .find(|c| c.name == "filter_output")
        .unwrap();
    assert!(filter_output
        .rules
        .contains(&"oif tailscale0 accept".to_string()));
}

#[test]
fn forward_chain_has_no_accept_rules_regardless_of_config() {
    // v0.1 never acts as a gateway/router for other devices — the
    // forward chain must stay a bare default-drop with zero exceptions.
    let mut config = default_config();
    config.excluded_tcp_ports.push(ExcludedTcpPort(80));
    config
        .excluded_interfaces
        .push(ExcludedInterface("eth1".to_string()));

    let ruleset = build_ruleset(&config);
    let forward = ruleset
        .chains
        .iter()
        .find(|c| c.name == "filter_forward")
        .unwrap();
    assert!(forward.rules.is_empty());
    assert_eq!(forward.hook.as_ref().unwrap().policy, "drop");
}

#[test]
fn no_accept_rule_exists_for_ipv6_in_block_mode() {
    // v0.1 hard-blocks all non-loopback IPv6 by simply never emitting an
    // accept rule for it — the chains' default-drop policy does the
    // rest. RouteThroughTor must not change this yet either (reserved
    // for a future release, see firewall::nft module docs).
    for ipv6_mode in [Ipv6Mode::Block, Ipv6Mode::RouteThroughTor] {
        let mut config = default_config();
        config.ipv6_mode = ipv6_mode;
        let ruleset = build_ruleset(&config);
        for chain in &ruleset.chains {
            for rule in &chain.rules {
                assert!(
                    !rule.contains("ip6"),
                    "unexpected ipv6 rule in {}: {rule}",
                    chain.name
                );
            }
        }
    }
}

#[test]
fn panic_ruleset_only_allows_loopback() {
    let ruleset = build_panic_ruleset();
    assert_eq!(ruleset.table_name, "anonveil_panic");
    for chain in &ruleset.chains {
        assert_eq!(chain.hook.as_ref().unwrap().policy, "drop");
        for rule in &chain.rules {
            assert!(rule.contains("lo accept"), "unexpected rule: {rule}");
        }
    }
}
