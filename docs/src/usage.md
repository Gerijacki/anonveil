# Usage

## Commands

| Command | Needs root | What it does |
|---|---|---|
| `anonveil start` | yes | Activate the kill switch: redirect all traffic through Tor. |
| `anonveil stop [--force]` | yes | Deactivate and restore your original network configuration. `--force` restores even if AnonVeil doesn't think it's active (e.g. after a crash). |
| `anonveil restart` | yes | Reapply the kill switch and Tor configuration from scratch. |
| `anonveil newnym` | no | Request a new Tor circuit/identity, without touching the firewall. |
| `anonveil status` | no | Show current protection state and live circuit info. |
| `anonveil panic` | yes | Instantly cut all traffic except loopback, unconditionally. |
| `anonveil check` | no | Confirm traffic is actually exiting through Tor right now. |
| `anonveil mac randomize [--interface <name>]` | yes | Randomize a network interface's MAC address. |
| `anonveil mac restore [--interface <name>]` | yes | Restore the original MAC address. |
| `anonveil` (no subcommand) | no | Launch the live-status TUI dashboard. |

## A typical session

```sh
sudo anonveil start
anonveil check
anonveil            # dashboard: press 'r' to refresh, 'q' to quit
sudo anonveil stop
```

## Panic mode

`anonveil panic` is deliberately more aggressive than `stop`: it loads
an unconditional drop-everything-but-loopback rule on top of whatever
state already exists, without trying to clean anything up gracefully
first. Use it when something looks wrong and you want connectivity cut
*right now*. Recovering from it always requires an explicit:

```sh
sudo anonveil stop --force
```

## Browsing `.onion` sites

Nothing extra to do — once `anonveil start` is active, any application
(a normal browser, `curl`, anything) can resolve and connect to
`.onion` addresses directly. See [the threat model](threat-model.md)
for what this does and doesn't mean for fingerprinting resistance.

## Excluding traffic from the kill switch

Edit `/etc/anonveil/config.toml`:

```toml
[network]
excluded_tcp_ports = [22]              # e.g. keep your own SSH session alive
excluded_interfaces = ["tailscale0"]   # e.g. don't block a VPN/mesh interface
```

Every entry here is a deliberate hole in the kill switch — see
[Configuration](configuration.md) and the threat model for exactly
what each one does and doesn't exempt.
