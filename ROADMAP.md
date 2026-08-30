# Roadmap

v0.1 is a complete, honest MVP: system-wide Tor transparent proxying,
a real kill switch, a CLI + TUI dashboard, and packaging for Arch and
Debian. Everything below is deliberately **not** in v0.1 — each item
is a scope decision explained in `threat-model.md`, not an oversight.

## Near-term

- **Per-circuit exit-relay detail in `status`/the TUI** (nickname/
  fingerprint of the current exit, not just bootstrap/circuit-established
  booleans and the bandwidth counters already shown). Needs
  `anonveil-core::control::protocol` to parse `+`-prefixed multi-line
  *data* replies (`GETINFO circuit-status`'s actual wire format), which
  it deliberately doesn't today — a real, scoped protocol extension, not
  a one-line addition.
- **IPv6 routed through Tor** (`ipv6_mode = "route_through_tor"`) —
  currently reserved in the config schema but behaves identically to
  `"block"`. Needs Tor's IPv6 SOCKS/TransPort support wired through
  `anonveil-core::firewall::nft` end-to-end, not just a config flag.
- **Automated AUR publishing** — release CI currently produces a
  PKGBUILD-ready source tarball but stops short of pushing to the AUR
  git repo, since that needs a maintainer's AUR SSH key as a repo
  secret. Wiring this up once that's available.
- **Signed release artifacts** — SHA256SUMS ships today; adding a
  minisign/cosign signature over it once a signing key exists.
- **Round-robin exit-country rotation** — `[rotation.ip]` today issues
  `NEWNYM` against whatever static `exit_nodes` config already says; a
  future version could cycle through multiple configured countries on
  each rotation instead of just one fixed set. Needs a torrc rewrite +
  `tor` reload per rotation (heavier and riskier than a plain `NEWNYM`
  signal), so it's deliberately scoped separately from the rotation
  system that shipped.

## Later

- **Per-application routing** — running only specific processes
  through Tor (via network namespaces/cgroups) instead of the whole
  host, for people who want AnonVeil active for one app without
  routing everything.
- **Gateway/router mode** — acting as a Tor gateway for other devices
  on a LAN (phone, another machine), the way Whonix-Gateway does. This
  needs the `prerouting`/`forward` nftables hooks the v0.1 ruleset
  deliberately omits (see `firewall::nft` module docs) plus IP
  forwarding, and is a meaningfully bigger security surface — not
  something to bolt on casually.
- **Optional desktop GUI** — a Tauri-based app reusing
  `anonveil-core`/`anonveil-priv`, for people who want a system-tray
  toggle instead of the terminal. The CLI/TUI stays the primary,
  first-class interface either way.
- **i2p support** as an alternative anonymity network alongside Tor.
- **Onion-service hosting helper** — a guided `anonveil onion serve
  <port>` for standing up a hidden service, complementing the
  "browse .onion" side AnonVeil already supports natively.

## Not planned

- Bundling or spawning the Tor daemon itself. AnonVeil manages the
  distro's `tor` package/service by design — see `threat-model.md` and
  `systemd.rs` — rather than vendoring Tor, which would duplicate a
  security-critical piece of software AnonVeil has no business
  maintaining a fork of.
- A "one-click" mode that silently changes what the kill switch
  exempts. Every exemption (`excluded_tcp_ports`,
  `excluded_interfaces`) stays an explicit, visible line in
  `config.toml` — see the "Deliberate v0.1 simplifications" section of
  `threat-model.md` for why that's a feature, not friction.

Have an idea not listed here? Open an issue — this file reflects
current intent, not a closed list.
