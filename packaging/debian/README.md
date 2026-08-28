# Debian packaging

AnonVeil's `.deb` metadata lives in [`crates/anonveil-cli/Cargo.toml`](../../crates/anonveil-cli/Cargo.toml)
under `[package.metadata.deb]`, built with [`cargo-deb`](https://github.com/kornelski/cargo-deb) —
there's no separate `debian/control` tree. This directory only holds the
maintainer scripts (`postinst`) referenced from that metadata.

## Building a `.deb` locally

```sh
cargo install cargo-deb
cargo build --release -p anonveil-cli
cargo deb -p anonveil-cli --no-build
```

The resulting package is written to `target/debian/`.

Release builds (`.github/workflows/release.yml`) run this automatically
on every tagged version and attach the result to the GitHub Release.
