## What does this change and why?

## Does this touch `anonveil-core::firewall` (the ruleset)?

- [ ] No.
- [ ] Yes — the golden-file test in `crates/anonveil-core/tests/firewall_rulegen.rs` is updated to match, and `threat-model.md` is updated if this changes what AnonVeil does or doesn't protect against.

## Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] If this changes privileged behavior (`anonveil-priv`), I've tested it manually against a real system, or explained why that's not feasible in this PR
