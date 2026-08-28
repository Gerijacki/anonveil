# FAQ

**Does this replace Tor Browser?**
No. AnonVeil anonymizes network traffic; it doesn't harden a browser
against fingerprinting. Use Tor Browser for browsing where
fingerprinting resistance matters, and AnonVeil as the system-wide
kill switch underneath everything else. See the
[threat model](../threat-model.md).

**Why not iptables?**
Both Arch's `iptables` package and Debian's default backend are
nftables-based today. AnonVeil targets nftables directly: one atomic,
isolated table (`inet anonveil`) that's trivial to add and remove
cleanly, instead of juggling `iptables`/`ip6tables`/`iptables-legacy`.

**Does it work with IPv6?**
IPv6 is hard-blocked by default in v0.1, not routed through Tor —
see the [threat model](../threat-model.md) for why, and the tradeoff
(a possible short connection delay for IPv6-preferring apps).

**Can I route my phone/another device through AnonVeil?**
Not in v0.1 — that's gateway/router mode, tracked in
[`ROADMAP.md`](https://github.com/Gerijacki/anonveil/blob/main/ROADMAP.md).
v0.1 only protects the host it's running on.

**Why does it need root?**
It loads firewall rules, rewrites `/etc/resolv.conf`, and manages the
`tor` systemd service — all privileged operations. `anonveil status`
and `anonveil check` don't need root.

**What happens if Tor crashes while AnonVeil is active?**
The kill switch stays up regardless — traffic doesn't fall back to the
clear network, it just stops working until Tor is back (or you run
`anonveil stop`). That's the point of a kill switch: it fails closed.

**Where are the logs?**
`/var/log/anonveil/`. Verbosity is set via `[logging] level` in
`config.toml`.

**Is this audited?**
Read the code — that's the honest answer for any young security tool.
The most security-critical logic
(`crates/anonveil-core/src/firewall/nft.rs`) is deliberately written as
a small, pure, heavily-commented function with a golden-file test
covering its exact output, specifically to make it easy to audit. See
[`CONTRIBUTING.md`](https://github.com/Gerijacki/anonveil/blob/main/CONTRIBUTING.md)
if you'd like to review it.
