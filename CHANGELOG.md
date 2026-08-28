# Changelog

All notable changes to this project are documented here. Format based
on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this
project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- **`stop`/`stop --force` could delete a live `/etc/resolv.conf`** if run
  without a genuine prior `start` having captured it (e.g. after only
  `anonveil panic`, or with a missing/reset state file) — the capture
  and "nothing to restore" cases were indistinguishable, and `stop`
  picked the wrong one. `StateSnapshot` now has an explicit
  `dns_snapshot_captured` flag; `stop` leaves `/etc/resolv.conf`
  untouched unless a real capture happened.
- **`start` closed the kill switch last** instead of first — a real
  (if brief) window existed on every activation where `/etc/resolv.conf`
  already pointed at Tor's `DNSPort` but the redirect/default-deny
  ruleset wasn't loaded yet, so a connection opened by any process in
  that window went out directly, unproxied, with the host's real
  source IP. The kill switch now loads before torrc/DNS are touched.
- **`start`/`restart` after a bare `anonveil panic`** (no prior
  `start`) failed with a confusing bootstrap-timeout instead of a clear
  error — the leftover `anonveil_panic` table blocks all traffic,
  including Tor's own, and nothing checked for it up front. `start` now
  detects this and tells the user to run `stop --force` first.
- **Tor control-port `PROTOCOLINFO` parsing truncated `COOKIEFILE`
  paths containing a space** (a naive space-split broke the quoted
  value apart), which would make authentication fail against an
  otherwise-valid, unusually-pathed `DataDirectory`.

### Added

- Initial release: `anonveil start|stop|restart|newnym|status|panic|check|mac`.
- System-wide Tor transparent proxying with a default-deny nftables
  kill switch (`crates/anonveil-core/src/firewall/`).
- Native `.onion`/`.exit` resolution via Tor's `AutomapHostsOnResolve`.
- First-party Tor control-port client (cookie auth, `SIGNAL NEWNYM`,
  `GETINFO`) — see `crates/anonveil-core/src/control/` for why this
  isn't a third-party dependency.
- Live-status TUI dashboard (`ratatui`).
- MAC address randomization (`anonveil mac randomize|restore`).
- Packaging for Arch (`packaging/arch/PKGBUILD`) and Debian/Ubuntu
  (`cargo-deb` via `crates/anonveil-cli/Cargo.toml`), plus a generic
  `install.sh`.
- CI: lint, tests on Ubuntu and an Arch container, a privileged
  nftables integration-test job, and a weekly supply-chain audit
  (`cargo-audit` + `cargo-deny`).
- `threat-model.md`: an explicit statement of what AnonVeil does and
  does not protect against.

[Unreleased]: https://github.com/Gerijacki/anonveil/compare/v0.1.0...HEAD
