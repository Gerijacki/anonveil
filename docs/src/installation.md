# Installation

AnonVeil needs `tor`, `nftables`, and `iproute2`. All three package
managers below pull these in automatically as dependencies.

## Arch Linux (AUR)

```sh
git clone https://aur.archlinux.org/anonveil.git
cd anonveil
makepkg -si
```

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
