# Manual testing checklist

CI verifies rule generation, unit tests, (in a privileged container)
that the real ruleset loads and unloads correctly against `nft`, and
(in the push-to-main `e2e-smoke-test` job) a full real
`start`/`check`/`stop` cycle against a real `tor` daemon. It cannot
verify an actual reboot, though (GitHub Actions runners are ephemeral —
there's nothing to reboot back into), so that and a few other
VM-only checks still need an actual Arch or Debian machine/VM with
`tor` installed and a real network connection. Run this checklist
before tagging a release.

## Setup

- A fresh Arch or Debian/Ubuntu VM (not a container — this needs a real
  systemd and a real network stack).
- `tor`, `nftables`, `iproute2` installed.
- AnonVeil built (`cargo build --release -p anonveil-cli`) or installed
  via the package for your distro.

## Checklist

1. **Baseline**: `curl https://check.torproject.org/api/ip` reports
   `"IsTor":false`. Note your real IP.
2. **Start**: `sudo anonveil start` completes without error.
3. **Ruleset loaded**: `sudo nft list table inet anonveil` shows the
   expected chains (`output`, `filter_input`, `filter_output`,
   `filter_forward`).
4. **DNS redirected**: `resolvectl status` or `cat /etc/resolv.conf`
   shows `nameserver 127.0.0.1`.
5. **Traffic actually goes through Tor**:
   `curl https://check.torproject.org/api/ip` now reports
   `"IsTor":true` and a different IP than the baseline.
6. **`.onion` works**: `curl -s http://expyuzz4wqqyqhjn.onion` (the Tor
   Project's own onion mirror) returns content, with no proxy flags
   passed to curl — confirming native `.onion` resolution.
7. **Kill switch actually blocks non-Tor traffic**: `sudo systemctl
   stop tor`, then confirm `curl -m 5 https://example.com` times out
   or fails (does **not** fall back to the clear network). Restart Tor
   afterward: `sudo systemctl start tor`.
8. **`anonveil status`** reports ACTIVE with bootstrap/circuit info.
9. **`anonveil check`** reports Tor-reachable.
10. **`anonveil newnym`** succeeds and (optionally) confirm the exit
    IP from step 5 changes.
11. **`anonveil panic`** immediately cuts all traffic — confirm even
    `ping 1.1.1.1` fails, and loopback (`curl localhost` against
    something local, if anything's listening) still works.
12. **`anonveil stop --force`** after panic cleanly restores
    connectivity: real IP is reachable again, `/etc/resolv.conf` is
    back to its pre-AnonVeil content, `nft list table inet anonveil`
    reports "No such file or directory".
13. **Clean `stop` without panic**: repeat steps 2–5, then
    `sudo anonveil stop` and confirm step 1's baseline behavior
    (`IsTor:false`, real IP, original `/etc/resolv.conf`) is fully
    restored.
14. **Dashboard**: `anonveil` with no arguments launches the TUI,
    shows live status, `r` refreshes, `q` exits cleanly (terminal
    state restored, no leftover alternate-screen garbage).
15. **MAC randomization** (if testing that feature):
    `sudo anonveil mac randomize` changes the interface's MAC
    (`ip link show <iface>`), `sudo anonveil mac restore` puts the
    original back.
16. **Reboot resilience**: with AnonVeil active, `sudo systemctl enable
    anonveil.service` then reboot the VM. After boot, `anonveil status`
    (before running `start` again) should show the kill switch reapplied
    automatically with no STATE MISMATCH warning, and step 5's
    Tor-reachability check should still pass. Then `sudo systemctl
    disable anonveil.service`, reboot again, and confirm this time
    `anonveil status` reports the STATE MISMATCH warning (kill switch
    gone, state.json still says active) instead of a false ACTIVE.
17. **Concurrent invocation is rejected, not racy**: with AnonVeil
    inactive, run `sudo anonveil start & sudo anonveil start; wait` —
    exactly one should succeed; the other should fail fast with "another
    `anonveil` operation is already in progress", not corrupt
    `state.json` or leave the firewall half-configured.

If any step fails, that's a release blocker — file an issue with which
step failed before tagging.
