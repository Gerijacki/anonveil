# Troubleshooting

Start here:

```sh
sudo anonveil doctor
```

It checks everything below in one pass and tells you exactly which one
failed, instead of you discovering them one `start` failure at a time.
This page explains what each failure mode actually means.

## `` `nft`/`tor`/`ip` not found on PATH ``

The corresponding package isn't installed. Arch: `sudo pacman -S nftables
tor iproute2`. Debian/Ubuntu: `sudo apt install nftables tor iproute2`.
(These are declared dependencies of both AnonVeil packages, so this
should only come up on a from-source install.)

## `tor systemd service not found`

Same fix as above — the `tor` package installs the systemd unit(s)
AnonVeil manages. If `tor` is installed but this still fails, check
`systemctl list-unit-files 'tor*'` — AnonVeil looks for
`tor@default.service` (Debian/Ubuntu's multi-instance unit) first, then
`tor.service` (Arch's, and older Debian's).

## `could not resolve the tor system user`

AnonVeil looks for a system user named `tor` (Arch) or `debian-tor`
(Debian/Ubuntu), created by the `tor` package's postinst. If neither
exists, reinstalling the `tor` package usually fixes it.

## `config.toml failed to parse`

`doctor` prints the parse error from `toml`/`serde` directly, which
names the exact key and line. Compare against
[`config/config.example.toml`](https://github.com/Gerijacki/anonveil/blob/main/config/config.example.toml) —
every key there is optional, so the most common cause is a typo'd key
name or a value of the wrong type (e.g. a bare string where a list is
expected).

## `<port> is already in use by something else`

Something other than Tor is already listening on AnonVeil's configured
`trans_port`/`dns_port`/`control_port` (defaults `9040`/`5353`/`9051`).
Either stop that process, or change the conflicting port in
`config.toml`'s `[network]` section — see [Configuration](configuration.md).

## `state.json says active, but the kill switch is NOT actually loaded`

nftables rules don't survive a reboot on their own. If you saw this
after a reboot, run `sudo anonveil start` to reapply it — or enable the
optional boot unit (`sudo systemctl enable anonveil.service`) so it
happens automatically next time. See [Usage](usage.md).

## `<dir> is not writable`

`doctor` (like `start`) needs root to create/write
`/var/lib/anonveil` and `/var/log/anonveil`. Run with `sudo`.

## `anonveil start` hangs at "waiting for tor to finish bootstrapping"

Up to 90 seconds is normal on a slow or first-time connection. If it
times out: check you actually have working internet (AnonVeil's kill
switch is already active at this point, so anything *other* than Tor is
already blocked — that's expected, not the problem). If bridges are
configured, confirm `obfs4proxy` is installed
(`anonveil doctor` checks this) and the bridge lines are current — public
bridge lines can go stale.

## `PANIC lockdown is engaged` when trying to `start`

Either `anonveil panic` was run and never cleared, or it was engaged
before a reboot and AnonVeil safely reapplied it rather than silently
resuming normal operation. Run `sudo anonveil stop --force` once it's
safe to restore connectivity.

## `another anonveil operation is already in progress`

Two `start`/`stop`/`panic` invocations overlapped. Wait for the first to
finish and retry — this is the lock (`anonveil-priv::lock`) doing its
job, not a bug.

## Still stuck?

Read the actual code that's failing — `anonveil-core`'s security-critical
logic is deliberately small and heavily commented, see
[Architecture](architecture.md) — or open an issue with `anonveil doctor`'s
full output attached.
