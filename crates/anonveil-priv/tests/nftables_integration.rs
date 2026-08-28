#![cfg(feature = "integration")]
//! Privileged integration tests: actually load/unload AnonVeil's real
//! nftables ruleset against a real `nft` binary and a real kernel
//! netfilter subsystem.
//!
//! Requires root and a working nftables stack. Run via
//! `cargo test -p anonveil-priv --features integration -- --test-threads=1`,
//! inside the isolated, privileged CI job (see `.github/workflows/ci.yml`)
//! — never against a machine, container, or network namespace you care
//! about, since these tests load and delete a real firewall table.
//!
//! `--test-threads=1` is not optional: every test here mutates the same
//! shared, global resource (the host's real nftables state), so running
//! them concurrently causes exactly the kind of cross-test interference
//! unit tests never have to think about — one test's `stop` can delete
//! a table another test just loaded. This was caught empirically, not
//! guessed: the default parallel run flakes.

use anonveil_core::firewall::FirewallConfig;

fn test_config() -> FirewallConfig {
    FirewallConfig {
        trans_port: 19040,
        dns_port: 15353,
        // "nobody" on virtually every Linux system — this test only
        // needs *a* valid uid to exercise the loop-prevention rule, not
        // an actual running Tor daemon.
        tor_uid: 65534,
        excluded_tcp_ports: Vec::new(),
        excluded_interfaces: Vec::new(),
        ipv6_mode: anonveil_core::firewall::Ipv6Mode::Block,
    }
}

/// Best-effort cleanup so a previous failed run doesn't cause a false
/// failure here.
fn ensure_clean_slate() {
    let _ = anonveil_priv::apply::teardown_main_ruleset();
    let _ = anonveil_priv::apply::teardown_panic();
}

#[test]
fn apply_and_teardown_main_ruleset_round_trips() {
    ensure_clean_slate();
    assert!(!anonveil_priv::apply::table_exists(
        "inet",
        FirewallConfig::TABLE_NAME
    ));

    anonveil_priv::apply::apply_main_ruleset(&test_config()).expect("apply should succeed");
    assert!(anonveil_priv::apply::table_exists(
        "inet",
        FirewallConfig::TABLE_NAME
    ));

    anonveil_priv::apply::teardown_main_ruleset().expect("teardown should succeed");
    assert!(!anonveil_priv::apply::table_exists(
        "inet",
        FirewallConfig::TABLE_NAME
    ));
}

#[test]
fn apply_refuses_when_table_already_exists() {
    ensure_clean_slate();
    anonveil_priv::apply::apply_main_ruleset(&test_config()).expect("first apply should succeed");

    let result = anonveil_priv::apply::apply_main_ruleset(&test_config());
    assert!(matches!(
        result,
        Err(anonveil_priv::PrivError::TableAlreadyExists)
    ));

    anonveil_priv::apply::teardown_main_ruleset().expect("cleanup");
}

#[test]
fn panic_ruleset_round_trips() {
    ensure_clean_slate();
    assert!(!anonveil_priv::apply::panic_active());

    anonveil_priv::apply::apply_panic().expect("panic apply should succeed");
    assert!(anonveil_priv::apply::panic_active());

    anonveil_priv::apply::teardown_panic().expect("panic teardown should succeed");
    assert!(!anonveil_priv::apply::panic_active());
}
