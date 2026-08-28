# Security policy

AnonVeil is a security- and privacy-critical tool: a bug in its
firewall rule generation or its network/DNS handling can mean a real
deanonymization or leak for someone relying on it. Please report
vulnerabilities responsibly.

## Reporting a vulnerability

**Do not open a public GitHub issue for a security vulnerability.**

Instead, use [GitHub's private vulnerability reporting](https://github.com/Gerijacki/anonveil/security/advisories/new)
for this repository. This opens a private draft security advisory
visible only to the maintainers until a fix is ready.

If that's not available to you for some reason, open a regular issue
that says only "I have a security report" with no details, and a
maintainer will follow up to arrange a private channel.

Please include:

- The affected version/commit.
- Whether the issue affects the firewall ruleset (`anonveil-core`'s
  `firewall::nft`), the Tor control-port client, DNS handling, or
  something else — and a link to the relevant code if you can point to
  it.
- Steps to reproduce, or a description of the failure mode if it's a
  design issue rather than a reproducible bug.
- What you believe the actual security impact is (e.g. "traffic
  bypasses the kill switch under condition X" vs. "this is a
  robustness bug, not a leak").

## Scope

In scope:

- Anything that could cause traffic to bypass the kill switch or leak
  outside Tor while AnonVeil reports itself as active.
- Anything that could cause a DNS leak.
- Privilege-escalation or command-injection issues in the privileged
  execution layer (`anonveil-priv`).
- Logic bugs in the Tor control-port client that could misreport
  status (e.g. reporting "bootstrapped" when it isn't).
- Supply-chain issues in a dependency AnonVeil ships.

Out of scope (see `threat-model.md` for why):

- Browser fingerprinting — that's Tor Browser's job, not AnonVeil's.
- Attacks that require the host itself to already be compromised.
- Issues that only affect the `RouteThroughTor` IPv6 mode, which is
  reserved/unimplemented in v0.1 (it currently behaves identically to
  `Block`).

## Disclosure

We'll acknowledge a report within a few days, work with you on a fix,
and credit you in the release notes (unless you'd rather stay
anonymous, which — this being AnonVeil — we obviously respect). We aim
to publish a fix and advisory together rather than delaying disclosure
indefinitely.
