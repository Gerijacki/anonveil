# Configuration

AnonVeil reads `/etc/anonveil/config.toml` if it exists; otherwise it
runs with built-in defaults (and prints a note saying so). Every
section and field is optional — an empty file is valid, and anything
you omit falls back to its default.

The full, documented example lives at
[`config/config.example.toml`](https://github.com/Gerijacki/anonveil/blob/main/config/config.example.toml)
in the repository; copy it to `/etc/anonveil/config.toml` as a
starting point.

## `[network]`

| Key | Default | Meaning |
|---|---|---|
| `trans_port` | `9040` | Loopback port Tor's `TransPort` listens on. |
| `dns_port` | `5353` | Loopback port Tor's `DNSPort` listens on. |
| `control_port` | `9051` | Loopback port Tor's `ControlPort` listens on. |
| `ipv6_mode` | `"block"` | `"block"` (only implemented mode in v0.1) hard-blocks non-loopback IPv6. `"route_through_tor"` is reserved for a future release — see the [threat model](threat-model.md). |
| `excluded_tcp_ports` | `[]` | TCP ports on this host that bypass the kill switch entirely. |
| `excluded_interfaces` | `[]` | Interfaces that bypass the kill switch's default-deny (does **not** exempt them from DNS/TCP redirection — see the threat model). |
| `exit_nodes` | `[]` | Two-letter country codes (e.g. `"us"`) or relay fingerprints to constrain exit circuits to. Empty means Tor's own default selection. |
| `exclude_exit_nodes` | `[]` | Country codes/fingerprints exit circuits must avoid. |
| `strict_exit_nodes` | `false` | Whether `exit_nodes`/`exclude_exit_nodes` is a hard requirement (Tor's `StrictNodes`) rather than a preference Tor may fall back from. |

**A note on `exit_nodes`/`exclude_exit_nodes`**: constraining which relays can be an exit shrinks the set of possible circuits — the same honesty principle as the rest of this project applies here too. Picking one specific country makes your traffic's exit more predictable to anyone who can see relay-selection statistics, and `strict_exit_nodes = true` can mean circuits fail to build at all if too few matching relays are available. Use the smallest constraint that actually solves your problem (e.g. one excluded country, not one required country) unless you have a specific reason not to.

## `[network.bridges]`

| Key | Default | Meaning |
|---|---|---|
| `enabled` | `false` | Whether to connect via pluggable-transport bridges instead of the public Tor network directly. |
| `lines` | `[]` | Raw `Bridge` lines, obtained from [bridges.torproject.org](https://bridges.torproject.org) or a trusted contact — AnonVeil never fetches or invents these. |

```toml
[network.bridges]
enabled = true
lines = ["obfs4 192.0.2.1:443 CERT=... IAT-MODE=0"]
```

Requires `obfs4proxy` installed (an optional dependency of both packages —
see [Installation](installation.md)); `anonveil doctor` checks for it when
bridges are enabled. Bridges hide *that* you're connecting to Tor at all
from a local network observer or censor — they don't add anonymity beyond
what Tor already provides once connected. See the [threat model](threat-model.md).

## `[mac]`

| Key | Default | Meaning |
|---|---|---|
| `randomize_on_start` | `false` | Randomize the default-route interface's MAC address every time `anonveil start` runs. |

## `[tui]`

| Key | Default | Meaning |
|---|---|---|
| `theme` | `"matrix"` | `"matrix"` (green) or `"cyan"`. |

## `[logging]`

| Key | Default | Meaning |
|---|---|---|
| `level` | `"info"` | `error` \| `warn` \| `info` \| `debug` \| `trace`. Logs go to `/var/log/anonveil/`. |

## Overriding the config path

```sh
sudo anonveil --config /path/to/other-config.toml start
```
