//! `anonveil doctor` — a single preflight/diagnostics command that
//! surfaces every precondition `start` depends on at once, instead of a
//! user discovering them one failure at a time. Never stops at the first
//! problem — every check runs and reports, and only the summary at the
//! end decides the exit code.

use std::net::TcpListener;
use std::path::PathBuf;

use anyhow::Result;

use crate::{config_paths, style};

struct Report {
    failed: u32,
    warned: u32,
}

impl Report {
    fn pass(&mut self, label: &str) {
        style::ok(label);
    }

    fn warn(&mut self, label: &str) {
        self.warned += 1;
        style::warn(label);
    }

    fn fail(&mut self, label: &str) {
        self.failed += 1;
        style::error(label);
    }
}

pub fn run(config_override: Option<&PathBuf>) -> Result<()> {
    let mut report = Report {
        failed: 0,
        warned: 0,
    };

    style::step("checking external commands...");
    for bin in ["nft", "tor", "ip"] {
        if anonveil_priv::exec::command_exists(bin) {
            report.pass(&format!("`{bin}` found on PATH"));
        } else {
            report.fail(&format!(
                "`{bin}` not found on PATH — install it before running `anonveil start`"
            ));
        }
    }

    style::step("checking the tor service...");
    if anonveil_priv::systemd::tor_service_installed() {
        report.pass("tor systemd service is installed");
    } else {
        report.fail(
            "tor systemd service not found (Arch: `sudo pacman -S tor`, \
             Debian: `sudo apt install tor`)",
        );
    }
    match anonveil_priv::privilege::resolve_tor_uid() {
        Ok(uid) => report.pass(&format!("`tor` system user resolved (uid {uid})")),
        Err(e) => report.fail(&format!("could not resolve the `tor` system user: {e}")),
    }

    style::step("checking configuration...");
    match config_paths::load(config_override) {
        Ok(config) => {
            report.pass("config.toml parses cleanly");

            if config.network.bridges.enabled && !anonveil_priv::exec::command_exists("obfs4proxy")
            {
                report.warn(
                    "bridges are enabled in config.toml but `obfs4proxy` isn't on PATH \
                     (Arch: `sudo pacman -S obfs4proxy`, Debian: `sudo apt install obfs4proxy`)",
                );
            }

            if config.rotation.ip.enabled && config.rotation.ip.interval_minutes < 1 {
                report.warn(
                    "[rotation.ip].interval_minutes is below Tor's own ~10s NEWNYM rate limit \
                     — most rotations this frequent would be wasted. See threat-model.md.",
                );
            }
            if config.rotation.mac.enabled && config.rotation.mac.interval_minutes < 5 {
                report.warn(
                    "[rotation.mac].interval_minutes is very short — each MAC rotation bounces \
                     the interface (brief connectivity interruption every time). See \
                     threat-model.md before leaving it this frequent.",
                );
            }

            style::step("checking whether AnonVeil's ports are free...");
            let active = anonveil_priv::snapshot::load_state()
                .map(|s| s.active)
                .unwrap_or(false);
            if active {
                style::dim(
                    "  AnonVeil is currently active, so its ports are expected to be bound — \
                     skipping.",
                );
            } else {
                for (label, port) in [
                    ("TransPort", config.network.trans_port),
                    ("DNSPort", config.network.dns_port),
                    ("ControlPort", config.network.control_port),
                ] {
                    match TcpListener::bind(("127.0.0.1", port)) {
                        Ok(listener) => {
                            drop(listener);
                            report.pass(&format!("{label} ({port}) is free"));
                        }
                        Err(e) => report.warn(&format!(
                            "{label} ({port}) is already in use by something else: {e} — \
                             `anonveil start` may fail until it's free"
                        )),
                    }
                }
            }
        }
        Err(e) => report.fail(&format!("config.toml failed to parse: {e:#}")),
    }

    style::step("checking for other firewall managers...");
    let other_firewalls: Vec<&str> = ["ufw", "firewalld"]
        .into_iter()
        .filter(|unit| anonveil_priv::exec::run_ok("systemctl", &["is-active", "--quiet", unit]))
        .collect();
    if other_firewalls.is_empty() {
        report.pass("no other firewall manager (ufw/firewalld) is active");
    } else {
        style::dim(&format!(
            "  {} active — AnonVeil's kill switch lives in its own isolated `inet anonveil` \
             nftables table by design and doesn't touch anyone else's rules, so this is \
             expected to coexist fine (e.g. on Omarchy, which ships ufw). Still worth a look \
             with `sudo nft list ruleset` after `start` if something seems off.",
            other_firewalls.join(", ")
        ));
    }

    style::step("checking persisted state against reality...");
    match anonveil_priv::snapshot::load_state() {
        Ok(state) if state.active => {
            if anonveil_priv::snapshot::kill_switch_actually_loaded() {
                report.pass("state.json says active, and the kill switch really is loaded");
            } else {
                report.warn(
                    "state.json says active, but the kill switch is NOT loaded right now \
                     (likely a reboot) — run `sudo anonveil start` to reapply it, or see \
                     `anonveil status`",
                );
            }
        }
        Ok(_) => report.pass("no active session (nothing to cross-check)"),
        Err(e) => report.warn(&format!("could not read state.json: {e}")),
    }

    style::step("checking writable paths...");
    for dir in ["/var/lib/anonveil", "/var/log/anonveil"] {
        match std::fs::create_dir_all(dir) {
            Ok(()) => report.pass(&format!("{dir} is writable")),
            Err(e) => report.fail(&format!("{dir} is not writable: {e} (run as root)")),
        }
    }

    println!();
    if report.failed > 0 {
        anyhow::bail!(
            "{} check(s) failed, {} warning(s) — fix the failures above before running \
             `anonveil start`",
            report.failed,
            report.warned
        );
    }
    if report.warned > 0 {
        style::warn(&format!(
            "all critical checks passed, {} warning(s) above worth a look",
            report.warned
        ));
    } else {
        style::ok("everything looks good.");
    }
    Ok(())
}
