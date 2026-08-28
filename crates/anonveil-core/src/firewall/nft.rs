//! Pure nftables ruleset generation.
//!
//! [`build_ruleset`] is the entire security model of AnonVeil, expressed
//! as a deterministic, side-effect-free function: same [`FirewallConfig`]
//! in, same [`NftRuleset`] out, every time. `anonveil-priv` is the only
//! crate that ever hands the *rendered* text of this ruleset to `nft`.
//!
//! ## Design notes (read before changing rule order — order is load-bearing)
//!
//! * **One isolated `inet anonveil` table.** `inet` covers IPv4 and IPv6
//!   in a single table, and using a dedicated table (rather than
//!   inserting rules into the host's existing chains) is what makes
//!   activation and teardown atomic and safe on a box that may already
//!   have its own nftables rules: `start` only ever adds this table,
//!   `stop` only ever deletes it.
//! * **`output` hook only, no `prerouting`/`forward`.** v0.1 protects
//!   *this host's own* traffic — it is not a gateway/router for other
//!   devices (that is a tracked ROADMAP item). Locally generated packets
//!   hit the `nat`/`output` hook before a routing decision is made, so
//!   that is the only NAT hook needed. `forward` stays `policy drop`
//!   with no accept rules, which is what keeps AnonVeil from
//!   accidentally acting as an open relay/gateway.
//! * **DNS is always redirected, with no local-subnet exemption.** Some
//!   transparent-proxy setups exempt "local" destinations from the DNS
//!   redirect; AnonVeil deliberately does not, because a resolver that
//!   happens to sit on a private IP is exactly the kind of leak this
//!   tool exists to prevent. Only `excluded_tcp_ports` create bypasses,
//!   and DNS (port 53) is not exemptable through that mechanism.
//! * **IPv6 is hard-blocked except loopback in v0.1** (`Ipv6Mode::Block`,
//!   the only implemented mode). No accept rule exists for it in the
//!   filter chains, so the chain's default-drop policy handles it. This
//!   is the safe default used by comparable tools; the tradeoff (clients
//!   that prefer IPv6 first may see a connect delay before falling back
//!   to IPv4) is documented in `threat-model.md`.
//! * **`skuid <tor_uid>` is the loop-prevention rule and must stay
//!   first** in both the NAT `output` chain and `filter_output`: without
//!   it, Tor's own outbound connections would be redirected back into
//!   its own `TransPort`, deadlocking the daemon.
//! * **`excluded_interfaces` only ever produces filter-chain accepts.**
//!   Matching `oif` reliably in the `nat`/`output` hook is not possible
//!   (routing/oif is not yet decided at that point), so an excluded
//!   interface is not exempted from the NAT redirect itself — only from
//!   the kill switch's default-deny. This is a known, documented
//!   limitation rather than a silent gap: see `threat-model.md`.

use super::types::{
    ExcludedInterface, ExcludedTcpPort, FirewallConfig, Ipv6Mode, NftChain, NftHook, NftRuleset,
    NftSet, NftSetElement,
};

const SET_LOCAL_NETS: &str = "local_nets";
const SET_LOCAL_NETS6: &str = "local_nets6";

/// Build the full AnonVeil nftables ruleset for the given configuration.
///
/// Pure function: performs no I/O and looks up nothing about the running
/// system. `config.tor_uid` in particular must be resolved by the caller
/// (`anonveil-priv`, via `getent passwd tor`) before this is called.
pub fn build_ruleset(config: &FirewallConfig) -> NftRuleset {
    NftRuleset {
        table_family: "inet",
        table_name: FirewallConfig::TABLE_NAME.to_string(),
        sets: vec![local_nets_set(), local_nets6_set()],
        chains: vec![
            nat_output_chain(config),
            filter_input_chain(config),
            filter_output_chain(config),
            filter_forward_chain(),
        ],
    }
}

fn local_nets_set() -> NftSet {
    NftSet {
        name: SET_LOCAL_NETS.to_string(),
        set_type: "ipv4_addr",
        interval: true,
        elements: FirewallConfig::default_local_nets_v4()
            .into_iter()
            .map(|s| NftSetElement(s.to_string()))
            .collect(),
    }
}

fn local_nets6_set() -> NftSet {
    NftSet {
        name: SET_LOCAL_NETS6.to_string(),
        set_type: "ipv6_addr",
        interval: true,
        elements: FirewallConfig::default_local_nets_v6()
            .into_iter()
            .map(|s| NftSetElement(s.to_string()))
            .collect(),
    }
}

fn nat_output_chain(config: &FirewallConfig) -> NftChain {
    let mut rules = Vec::new();

    // Loop prevention: Tor's own traffic must never be redirected into
    // itself. Must be the first rule in this chain.
    rules.push(format!("skuid {} return", config.tor_uid));

    // Explicit per-port bypasses (e.g. an admin's own SSH session) skip
    // the NAT redirect entirely, before the general TCP/DNS rules below.
    for ExcludedTcpPort(port) in &config.excluded_tcp_ports {
        rules.push(format!("tcp dport {port} return"));
    }

    // DNS: always redirected, both protocols, no destination exemption.
    rules.push(format!(
        "meta l4proto {{ tcp, udp }} th dport 53 redirect to :{}",
        config.dns_port
    ));

    // Direct LAN access (printers, routers, local dev servers, ...)
    // bypasses Tor for plain TCP once DNS has already been captured
    // above.
    rules.push(format!("ip daddr @{SET_LOCAL_NETS} return"));

    // Everything else: transparently redirect to Tor's TransPort.
    rules.push(format!(
        "meta l4proto tcp redirect to :{}",
        config.trans_port
    ));

    NftChain {
        name: "output".to_string(),
        hook: Some(NftHook {
            chain_type: "nat",
            hook: "output",
            priority: "-100",
            policy: "accept",
        }),
        rules,
    }
}

fn filter_input_chain(_config: &FirewallConfig) -> NftChain {
    let rules = vec![
        "ct state { established, related } accept".to_string(),
        "iif lo accept".to_string(),
        format!("ip saddr @{SET_LOCAL_NETS} accept"),
    ];

    NftChain {
        name: "filter_input".to_string(),
        hook: Some(NftHook {
            chain_type: "filter",
            hook: "input",
            priority: "filter",
            policy: "drop",
        }),
        rules,
    }
}

fn filter_output_chain(config: &FirewallConfig) -> NftChain {
    let mut rules = vec![
        "ct state { established, related } accept".to_string(),
        "oif lo accept".to_string(),
        format!("skuid {} accept", config.tor_uid),
    ];

    for ExcludedInterface(iface) in &config.excluded_interfaces {
        rules.push(format!("oif {iface} accept"));
    }
    for ExcludedTcpPort(port) in &config.excluded_tcp_ports {
        rules.push(format!("tcp dport {port} accept"));
    }

    rules.push(format!("ip daddr @{SET_LOCAL_NETS} accept"));
    // Let the traffic AnonVeil itself just redirected reach Tor's local
    // listeners; everything else falls through to the chain's drop policy.
    rules.push(format!(
        "udp dport {} ip daddr 127.0.0.1 accept",
        config.dns_port
    ));
    rules.push(format!(
        "tcp dport {} ip daddr 127.0.0.1 accept",
        config.trans_port
    ));

    if config.ipv6_mode == Ipv6Mode::RouteThroughTor {
        // Reserved for a future release; intentionally not implemented
        // in v0.1 (see module docs and ROADMAP.md). Falling through to
        // the same hard-block behavior as `Block` is deliberate: no
        // half-correct IPv6 accept rule ships until the corresponding
        // Tor-side routing is implemented end-to-end.
    }

    NftChain {
        name: "filter_output".to_string(),
        hook: Some(NftHook {
            chain_type: "filter",
            hook: "output",
            priority: "filter",
            policy: "drop",
        }),
        rules,
    }
}

fn filter_forward_chain() -> NftChain {
    NftChain {
        name: "filter_forward".to_string(),
        hook: Some(NftHook {
            chain_type: "filter",
            hook: "forward",
            priority: "filter",
            policy: "drop",
        }),
        rules: Vec::new(),
    }
}

/// A separate, minimal ruleset for `anonveil panic`: default-drop
/// everything except loopback, on top of whatever table already exists.
/// Intentionally does not touch/replace the main `inet anonveil` table
/// (see `anonveil-priv::apply` for why: panic must be instant and must
/// not depend on cleanly tearing anything down first).
pub fn build_panic_ruleset() -> NftRuleset {
    NftRuleset {
        table_family: "inet",
        table_name: "anonveil_panic".to_string(),
        sets: Vec::new(),
        chains: vec![
            NftChain {
                name: "panic_input".to_string(),
                hook: Some(NftHook {
                    chain_type: "filter",
                    hook: "input",
                    priority: "filter",
                    policy: "drop",
                }),
                rules: vec!["iif lo accept".to_string()],
            },
            NftChain {
                name: "panic_output".to_string(),
                hook: Some(NftHook {
                    chain_type: "filter",
                    hook: "output",
                    priority: "filter",
                    policy: "drop",
                }),
                rules: vec!["oif lo accept".to_string()],
            },
            NftChain {
                name: "panic_forward".to_string(),
                hook: Some(NftHook {
                    chain_type: "filter",
                    hook: "forward",
                    priority: "filter",
                    policy: "drop",
                }),
                rules: Vec::new(),
            },
        ],
    }
}
