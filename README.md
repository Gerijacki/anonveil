<div align="center">

```
   ▄▀█ █▄░█ █▀█ █▄░█ █░█ █▀▀ █ █░░
   █▀█ █░▀█ █▄█ █░▀█ ▀▄▀ ██▄ █ █▄▄
```

**A system-wide Tor kill switch for Arch and Debian.**
One command anonymizes every process on the machine — and if Tor goes
down, AnonVeil fails *closed*, not open.

[![CI](https://github.com/Gerijacki/anonveil/actions/workflows/ci.yml/badge.svg)](https://github.com/Gerijacki/anonveil/actions/workflows/ci.yml)
[![Supply chain audit](https://github.com/Gerijacki/anonveil/actions/workflows/audit.yml/badge.svg)](https://github.com/Gerijacki/anonveil/actions/workflows/audit.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Latest release](https://img.shields.io/github/v/release/Gerijacki/anonveil?include_prereleases)](https://github.com/Gerijacki/anonveil/releases)

[Docs](https://gerijacki.github.io/anonveil/) · [Threat model](threat-model.md) · [Roadmap](ROADMAP.md) · [Security policy](SECURITY.md)

</div>

---

## What it does

```console
$ sudo anonveil start
:: writing torrc fragment and reloading tor...
:: pointing DNS at Tor...
:: loading kill switch...
:: waiting for tor to finish bootstrapping (this can take a moment)...
✔ AnonVeil is active — all traffic is now routed through Tor.

$ anonveil check
:: checking Tor reachability via check.torproject.org...
✔ Traffic from this host is exiting through Tor.
  exit relay IP: 51.15.0.1
```

AnonVeil redirects **every** outbound TCP connection and **every** DNS
query on the host through Tor, and drops anything that doesn't go
through it — a real kill switch, not a "route what I remembered to
configure." `.onion` addresses resolve natively the moment it's
active, with zero extra setup. When you're done: `anonveil stop`
restores your exact previous firewall and DNS configuration, byte for
byte.

It installs on the Arch or Debian system you already have — no
separate OS, no VM, no dedicated hardware.

## Features

- **`start` / `stop` / `restart`** — activate, deactivate, and
  cleanly restore your original network configuration.
- **`panic`** — an instant, unconditional kill switch: cut everything
  but loopback right now, independent of whatever state AnonVeil
  thinks it's in.
- **`newnym`** — request a fresh Tor circuit/identity without touching
  the firewall.
- **`status` / `check`** — live circuit info and a real
  Tor-reachability self-test.
- **`mac randomize` / `mac restore`** — optional MAC address spoofing.
- **A live TUI dashboard** — run `anonveil` with no arguments.
- **Native `.onion` browsing** — Tor's own hostname-mapping resolves
  `.onion`/`.exit` addresses without a real DNS lookup ever happening.
- **Honest documentation** — see [`threat-model.md`](threat-model.md)
  for exactly what this does and doesn't protect against. It's not a
  substitute for Tor Browser's anti-fingerprinting; it's a complement
  to it.

## Install

**Arch (AUR):**

```sh
git clone https://aur.archlinux.org/anonveil.git && cd anonveil && makepkg -si
```

**Debian/Ubuntu (.deb):**

```sh
curl -fsSL https://github.com/Gerijacki/anonveil/releases/latest/download/anonveil_amd64.deb -o anonveil.deb
sudo dpkg -i anonveil.deb || sudo apt-get install -f
```

**Anything else (prebuilt binary):**

```sh
curl -fsSL https://raw.githubusercontent.com/Gerijacki/anonveil/main/packaging/install.sh | sudo sh
```

**From source:**

```sh
git clone https://github.com/Gerijacki/anonveil.git && cd anonveil
cargo build --release -p anonveil-cli
sudo install -Dm755 target/release/anonveil /usr/local/bin/anonveil
```

Requires `tor`, `nftables`, and `iproute2` — installed automatically as
package dependencies, or install them yourself if building from source.

## Quickstart

```sh
sudo anonveil start     # activate the kill switch
anonveil check           # confirm traffic is exiting through Tor
anonveil                 # live dashboard (Ctrl+C / q to exit)
sudo anonveil stop        # restore your original configuration
```

Configuration lives at `/etc/anonveil/config.toml` — see
[`config/config.example.toml`](config/config.example.toml) for every
option, documented inline.

## How it works

```
                 ┌─────────────────────────────────────────┐
                 │              this host                  │
                 │                                          │
  every app ───► │  nftables (inet anonveil table)          │
                 │    │                                      │
                 │    ├─ port 53 (DNS)  ──► redirect ──┐     │
                 │    ├─ TCP            ──► redirect ──┤     │
                 │    └─ everything else ──► DROP      │     │
                 │                                      ▼     │
                 │                                tor daemon  │
                 │                          (DNSPort/TransPort)│
                 └─────────────────────────────────────┬─────┘
                                                          │
                                                     Tor network
```

Three crates, split strictly by privilege:

- [`anonveil-core`](crates/anonveil-core) — pure logic: nftables rule
  generation, torrc generation, the Tor control-port protocol, config
  parsing. No root, no I/O side effects — fully unit-tested with no
  privileges required.
- [`anonveil-priv`](crates/anonveil-priv) — the only crate that
  touches the OS: runs `nft`/`systemctl`/`ip`, manages
  `/etc/resolv.conf`, opens the real control-port socket.
- [`anonveil-cli`](crates/anonveil-cli) — the `anonveil` binary: CLI
  and TUI.

The full design rationale — why nftables over iptables, why the Tor
control-port client is first-party, exact chain/rule structure — is
documented in [`crates/anonveil-core/src/firewall/nft.rs`](crates/anonveil-core/src/firewall/nft.rs)
and [`crates/anonveil-core/src/control/mod.rs`](crates/anonveil-core/src/control/mod.rs).

## How it compares

| | AnonVeil | Tails | Whonix | AnonSurf / nipe |
|---|---|---|---|---|
| Installs on your existing OS | ✅ | ❌ (boot media) | ❌ (VM pair) | ✅ |
| Real kill switch (fails closed) | ✅ | N/A (whole OS is routed) | ✅ | ✅ |
| Native `.onion` resolution | ✅ | ✅ | ✅ | Varies |
| Live TUI dashboard | ✅ | ❌ | ❌ | ❌ |
| Explicit threat model in the docs | ✅ | ✅ | ✅ | Varies |
| Anti-fingerprinting browser included | ❌ (pair with Tor Browser) | ✅ | ✅ | ❌ |

Tails and Whonix anonymize an entire operating environment and include
a hardened browser — reach for them when that's what you need. AnonVeil
is for anonymizing the Linux box you're already running, with a kill
switch you can audit, on a system you don't have to rebuild.

## Documentation

- [Threat model](threat-model.md) — what AnonVeil does and doesn't protect against. Read this first.
- [Full documentation site](https://gerijacki.github.io/anonveil/)
- [Roadmap](ROADMAP.md)
- [Manual testing checklist](MANUAL_TESTING.md) (used before every release)

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) — in particular, read it
before touching `crates/anonveil-core/src/firewall/nft.rs`, which is
the actual security model of this tool.

## Security

Found a vulnerability? See [`SECURITY.md`](SECURITY.md) — please don't
open a public issue for it.

## License

[GPL-3.0-or-later](LICENSE).
