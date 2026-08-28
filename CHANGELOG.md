# Changelog

All notable changes to this project are documented here. Format based
on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this
project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
