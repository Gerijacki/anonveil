# Contributing to AnonVeil

Thanks for considering it. This is security-sensitive software, so the
bar for the firewall/network-handling code is higher than "it works on
my machine" — read on before opening a PR there.

## Project layout

- `crates/anonveil-core` — pure logic (nftables rule generation, torrc
  generation, the Tor control-port protocol, config parsing, state
  types). No root, no file I/O, no process spawning. Runs anywhere,
  including in ordinary CI with no privileges.
- `crates/anonveil-priv` — the privileged execution layer: the only
  crate that shells out to `nft`/`systemctl`/`ip` or touches
  `/etc`, `/run`, `/var/lib/anonveil`. Linux-only by design.
- `crates/anonveil-cli` — the `anonveil` binary: CLI parsing (clap),
  the TUI dashboard (ratatui), logging, config loading.

If your change could be expressed as a pure function, it belongs in
`anonveil-core`, not `anonveil-priv` or `anonveil-cli` — that's what
keeps it unit-testable without root.

## Building and testing

```sh
cargo build --workspace
cargo test --workspace          # anonveil-core + anonveil-priv's
                                 # unit tests need no privileges
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

`anonveil-priv` and `anonveil-cli` only compile on Linux (they use
`nix`, `std::os::unix`, and shell out to Linux-only tools). If you're
on Windows or macOS, you can still fully develop and test
`anonveil-core` locally; use a Linux VM/container or rely on CI for
the other two crates.

### Privileged integration tests

`crates/anonveil-priv/tests/nftables_integration.rs` actually loads and
deletes a real nftables table. It's feature-gated and does not run by
default:

```sh
# only inside a disposable container/VM you don't mind mutating the
# firewall state of — never on a machine you care about
cargo test -p anonveil-priv --features integration -- --test-threads=1
```

`--test-threads=1` is required — these tests share real, global
firewall state, and running them concurrently causes real
cross-test interference (this was found empirically, not assumed).

## Changing the firewall ruleset

`crates/anonveil-core/src/firewall/nft.rs` is the security model of
this tool. If you're changing rule order, adding a new chain, or
changing what gets exempted from the kill switch:

1. Read the module-level doc comment in `nft.rs` first — rule order is
   load-bearing and the rationale for each rule is documented there.
2. Add or update the golden-file test in
   `crates/anonveil-core/tests/firewall_rulegen.rs` — it asserts the
   *exact* rendered `nft` script text, so a change in behavior should
   show up as an intentional, reviewed diff to that test, not a
   silent change.
3. If you can, verify the change actually loads against a real `nft`
   binary (see the integration tests above) before opening the PR.
4. Update `threat-model.md` if the change affects what AnonVeil does
   or doesn't protect against.

## Regenerating the docs/README screenshots

`docs/assets/{hero.gif,dashboard.png,help.png,social-preview.png}` are real recordings of
the actual binary (via [VHS](https://github.com/charmbracelet/vhs)), not
mockups — see `docs/vhs/*.tape` for the scripts. Regenerate them when a
CLI/TUI change would make them stale (before a release, or after any
visible output change):

```sh
docker run --rm --privileged --cap-add=NET_ADMIN \
  -v "$PWD:/workspace" -w /workspace \
  rust:1-bookworm ./scripts/generate-screenshots.sh
```

Needs root, a real network, and a headless-Chromium-based recorder, so
it's not run in CI — see the script's own comments for why, and for the
handful of environment-only accommodations it makes for running inside a
plain container (no systemd, Docker's own `/etc/resolv.conf` bind-mount)
that a real target host never needs.

## Commit/PR expectations

- Keep the "why", not just the "what", in commit messages and PR
  descriptions — especially for anything touching `anonveil-priv` or
  the firewall rules.
- CI (lint, tests on Ubuntu and Arch, supply-chain audit) must pass.
- New dependencies: prefer ones already used elsewhere in the
  workspace; anything touching `anonveil-core` or `anonveil-priv`
  should have a real justification (see `deny.toml` and the rationale
  in `crates/anonveil-core/src/control/mod.rs` for why the Tor
  control-port client is first-party instead of a dependency, as an
  example of the bar).

## Code of conduct

See [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).
