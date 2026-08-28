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
