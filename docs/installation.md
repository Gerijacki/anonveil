# Installation

AnonVeil needs `tor`, `nftables`, and `iproute2`. All three package
managers below pull these in automatically as dependencies.

## Arch Linux (AUR)

```sh
git clone https://aur.archlinux.org/anonveil.git
cd anonveil
makepkg -si
```

### On Omarchy

[Omarchy](https://omarchy.org) is Arch underneath (pacman, AUR, systemd),
so the install above applies as-is. Two Omarchy-specific defaults worth
knowing:

- **NetworkManager** is Omarchy's network manager. This matters mainly
  for `[rotation.mac]` (see [Usage](usage.md)) — the interface bounce a
  MAC rotation requires can make NetworkManager notice a "new" link and
  momentarily reassert control over `/etc/resolv.conf`; `anonveil rotate
  --mac` already defends against this (see
  [the threat model](../threat-model.md)), but it's the concrete reason
  that caveat exists, not a hypothetical. The same applies to Omarchy's
  own `omarchy dns`/`omarchy network` commands — avoid running them while
  AnonVeil is active, same as you would with `nmcli`/`nmtui` directly.
- **ufw** ships with Omarchy (state — enabled or not — has varied across
  Omarchy versions; check with `systemctl status ufw`). This has been
  verified to coexist cleanly with AnonVeil: on Arch, `ufw` manages its
  rules through `ip filter`/`ip6 filter` tables via the `iptables-nft`
  compatibility layer, completely separate from AnonVeil's own isolated
  `inet anonveil` table — loading AnonVeil's ruleset alongside an active
  `ufw` and then tearing it back down again leaves `ufw`'s tables and
  status untouched, confirmed directly against real `nft`/`ufw`, not
  assumed. `anonveil doctor` flags when `ufw`/`firewalld` is active as an
  informational note, not a failure.

## Debian / Ubuntu (.deb)

```sh
curl -fsSL https://github.com/Gerijacki/anonveil/releases/latest/download/anonveil_amd64.deb -o anonveil.deb
sudo dpkg -i anonveil.deb || sudo apt-get install -f
```

## Any other distro (prebuilt binary)

```sh
curl -fsSL https://raw.githubusercontent.com/Gerijacki/anonveil/main/packaging/install.sh | sudo sh
```

This detects Arch/Debian automatically and defers to the instructions
above; on anything else it downloads a checksummed prebuilt binary from
the latest [GitHub release](https://github.com/Gerijacki/anonveil/releases).

## From source

```sh
git clone https://github.com/Gerijacki/anonveil.git
cd anonveil
cargo build --release -p anonveil-cli
sudo install -Dm755 target/release/anonveil /usr/local/bin/anonveil
sudo mkdir -p /etc/anonveil
sudo cp config/config.example.toml /etc/anonveil/config.toml
```

You'll need `tor` and `nftables` installed separately in this case —
they're not build-time dependencies, only runtime ones.

## Verifying the install

```sh
anonveil --version
sudo anonveil start
anonveil check
```

If `anonveil check` reports Tor-reachable, you're set. See
[Usage](usage.md) for the rest of the commands.
