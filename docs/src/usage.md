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
| `anonveil doctor` | no | Check every precondition `start` depends on at once. |
| `anonveil audit-ruleset` | no | Print the exact nftables ruleset `start` would load, without touching the system. |
| `anonveil completions <shell>` | no | Print a shell completion script (installed automatically by both packages). |
| `anonveil man` | no | Print the man page (installed automatically by both packages). |
| `anonveil` (no subcommand) | no | Launch the live-status TUI dashboard. |

## The dashboard is a control panel, not just a status light

Press `[s]` to start/stop, `[p]` for panic, `[n]` for a new circuit, `[r]`
to refresh, `[q]` to quit — right from `anonveil` with no arguments.
Privileged actions (`s`/`p`) leave the dashboard for their normal,
step-by-step output (starting can take up to 90s waiting for Tor to
bootstrap — worth seeing live, not hidden behind a spinner) and return you
to the dashboard afterward.

## Before you start: `anonveil doctor`

```sh
anonveil doctor
```

Checks `nft`/`tor`/`ip` are installed, the `tor` service exists, your
config parses, AnonVeil's ports are free, and more — all at once, instead
of discovering each precondition one failed `start` at a time.

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

## Using bridges

If plain Tor is blocked on your network, enable obfs4 pluggable-transport
bridges in `/etc/anonveil/config.toml`:

```toml
[network.bridges]
enabled = true
lines = ["obfs4 192.0.2.1:443 CERT=... IAT-MODE=0"]
```

Get bridge lines from [bridges.torproject.org](https://bridges.torproject.org)
or a trusted contact — AnonVeil never fetches or invents them. Requires
`obfs4proxy` (see [Installation](installation.md)); `anonveil doctor`
checks for it. See [Configuration](configuration.md) for what bridges do
and don't add to your anonymity.

## Choosing exit nodes

```toml
[network]
exit_nodes = ["us"]           # or exclude_exit_nodes = ["cn"]
strict_exit_nodes = false     # true = hard requirement, may fail to connect
```

Read the note in [Configuration](configuration.md) before setting this —
constraining exit nodes shrinks your anonymity set.

## Auditing the exact ruleset before you trust it

```sh
anonveil audit-ruleset
```

Prints the *exact* nftables script `start` would load for your current
config — no root required, nothing is applied. Read it, or diff it after
a config change, before running `start` for real.
