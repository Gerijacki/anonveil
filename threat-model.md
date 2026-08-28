# Threat model

This document says plainly what AnonVeil protects against, what it does
not, and the deliberate simplifications v0.1 makes. Being precise here
matters more than sounding impressive — if you're relying on this tool
for something where the difference matters, read this in full first.

## What AnonVeil actually does

AnonVeil makes **all outbound network traffic from this host** go
through Tor, and **blocks anything that doesn't**:

- Every outbound TCP connection is transparently redirected to Tor's
  `TransPort`.
- Every DNS query (port 53, TCP and UDP) is redirected to Tor's
  `DNSPort` — nothing on this host resolves a hostname over the clear
  network while AnonVeil is active.
- Everything else — every other protocol, every other destination —
  is dropped by a default-deny nftables ruleset (the "kill switch"),
  not merely left unrouted. If Tor is down, traffic doesn't silently
  fall back to the clear network; it stops.
- `.onion` (and `.exit`) addresses resolve natively, with no extra
  configuration, because Tor is asked to map them internally
  (`AutomapHostsOnResolve`) instead of ever attempting a real DNS
  lookup for them.

This gives you **network-level anonymity and a kill switch** for
everything running on this machine: your browser, your package
manager, your other CLI tools, all of it — not just one application
you remembered to configure.

## What AnonVeil does *not* do

**It does not make your browser fingerprint-resistant.** Tor Browser's
anonymity comes from two things working together: routing traffic
through Tor, *and* a browser hardened and standardized specifically to
resist fingerprinting (canvas/WebGL fingerprinting, font enumeration,
screen-size correlation, timezone/locale leaks, extension fingerprints,
and so on). AnonVeil only does the first half. Point an ordinary
browser at the internet through AnonVeil and its IP is anonymized —
but its fingerprint is still exactly as unique and identifying as it
was before.

**If fingerprinting resistance matters for what you're doing, use Tor
Browser.** It works seamlessly on top of AnonVeil (or on its own), and
AnonVeil's transparent, system-wide kill switch is a reasonable
complement to it — a safety net for everything else on the machine
while Tor Browser handles the part that needs real anti-fingerprinting.

**It does not protect against a compromised host.** If malware or an
attacker already has code execution on this machine, no network-layer
tool can anonymize you — they can read your keys, your screen, your
memory. AnonVeil's threat model assumes the host itself is trusted.

**It does not protect against application-layer deanonymization.**
Logging into an account tied to your identity, uploading a file with
identifying metadata, or being fingerprinted by unique behavior all
work exactly as well over Tor as off it. Anonymizing the network layer
doesn't anonymize you.

## Deliberate v0.1 simplifications

These are documented tradeoffs, not oversights:

- **IPv6 is hard-blocked, not routed through Tor.** `ipv6_mode` in
  `config.toml` currently only supports `"block"` — all non-loopback
  IPv6 traffic is dropped outright rather than partially/incorrectly
  tunneled. The tradeoff: an application that prefers IPv6 first (the
  common "happy eyeballs" pattern) may see a short connection delay
  before it falls back to IPv4. Routing IPv6 through Tor properly is
  tracked in `ROADMAP.md`.
- **UDP other than DNS is dropped, not proxied.** Tor cannot
  transparently proxy arbitrary UDP, so anything relying on it (raw
  UDP protocols, some VPN/game traffic, WebRTC's UDP path) is simply
  blocked rather than silently leaking over the clear network. This is
  the safe default; punching a UDP-shaped hole in a kill switch is not
  a tradeoff AnonVeil makes for you automatically.
- **`excluded_interfaces` does not exempt an interface from Tor's own
  DNS/TCP redirection**, only from the kill switch's default-deny. The
  nftables NAT redirect (which sends port-53/TCP traffic to Tor) runs
  at a hook where the outbound interface isn't yet reliably knowable —
  see the rule-generation design notes in
  `crates/anonveil-core/src/firewall/nft.rs` for the technical detail.
  In practice: adding an interface here stops AnonVeil from blocking
  its traffic; it does not stop AnonVeil from routing that traffic's
  DNS/TCP through Tor first.
- **Cookie authentication only** for the Tor control port. Password
  authentication isn't implemented. This matches how AnonVeil manages
  Tor (`CookieAuthentication 1` in the torrc fragment it writes) and
  keeps the control-port client simpler and more auditable — see
  `crates/anonveil-core/src/control/`.
- **nftables rules do not survive a reboot on their own.** If AnonVeil is
  active and the host reboots, the kill switch is gone until something
  reapplies it — `state.json` will still say `active: true`, but
  `anonveil status`/the TUI now detect and loudly flag that mismatch
  rather than reporting a false ACTIVE. An **opt-in** `anonveil.service`
  systemd unit (installed by both packages, never enabled automatically)
  resumes a previously-active session at boot for anyone who wants that;
  see `packaging/systemd/anonveil.service`. It deliberately stays opt-in
  rather than default-on — see `ROADMAP.md`'s "Not planned" reasoning on
  silent behavior changes, which applies here too.
- **Bridges hide that you're using Tor, not who you are once connected.**
  `[network.bridges]` (obfs4 pluggable transports) makes your connection
  to the Tor network harder for a local network observer or censor to
  identify as Tor traffic. It does not add anonymity beyond what Tor
  already provides once the connection is established — the rest of this
  threat model applies identically whether bridges are in use or not.
  Constraining `exit_nodes`/`exclude_exit_nodes` has the opposite kind of
  cost: it shrinks the pool of possible exit relays, which is a tradeoff
  against anonymity, not for it — see `configuration.md`.
- **v0.1 does not act as a gateway/router for other devices.** Only
  this host's own traffic is protected. Routing a phone or another
  machine's traffic through an AnonVeil host is a tracked `ROADMAP.md`
  item, not something to attempt by hand-editing the firewall rules —
  the current ruleset is deliberately scoped (no `prerouting`/`forward`
  handling) to avoid a half-correct version of that feature shipping
  by accident.

## `anonveil check`'s actual scope

`anonveil check` asks Tor Project's own check endpoint
(`https://check.torproject.org/api/ip`) whether the connection it just
received came from a Tor exit relay. Reaching that endpoint at all
already exercises both the DNS and TCP redirection paths for real
(since the request itself is subject to the same kill switch as
everything else on the host). A `true` result is a meaningful,
honest signal that AnonVeil's redirection is working right now.

What it is *not* is a full DNS-leak-testing tool in the sense some
commercial services mean: it doesn't enumerate every resolver on the
public internet checking who saw your query. That kind of leak,
structurally, isn't possible against AnonVeil's design in the first
place — the firewall drops any DNS packet that isn't going to Tor's
own `DNSPort` on loopback, by construction, before it would ever reach
a real resolver. The kill switch is the leak-prevention mechanism;
`anonveil check` is a reachability confirmation, not a substitute for
reading the ruleset.

## Reporting a security issue

See [`SECURITY.md`](https://github.com/Gerijacki/anonveil/blob/main/SECURITY.md).
