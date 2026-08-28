#!/bin/sh
# AnonVeil installer.
#
#   curl -fsSL https://raw.githubusercontent.com/Gerijacki/anonveil/main/packaging/install.sh | sh
#
# Prefers your distro's native package manager (so you get proper
# uninstall/upgrade support); falls back to downloading a prebuilt
# binary from the latest GitHub Release and verifying its checksum.
set -eu

REPO="Gerijacki/anonveil"
BIN_DIR="/usr/local/bin"

log() { printf '==> %s\n' "$1"; }
die() { printf 'error: %s\n' "$1" >&2; exit 1; }

if [ "$(id -u)" -ne 0 ]; then
    die "this installer needs root (it installs a binary and creates /etc/anonveil) — re-run with sudo"
fi

if [ -f /etc/arch-release ]; then
    log "Arch Linux detected."
    log "AnonVeil is best installed from the AUR:"
    log "  git clone https://aur.archlinux.org/anonveil.git && cd anonveil && makepkg -si"
    log "(this script does not build AUR packages for you — see README.md)"
    exit 0
fi

if [ -f /etc/debian_version ]; then
    log "Debian/Ubuntu detected. Downloading the latest .deb from GitHub Releases..."
    ARCH="$(dpkg --print-architecture)"
    URL="https://github.com/${REPO}/releases/latest/download/anonveil_amd64.deb"
    if [ "$ARCH" != "amd64" ]; then
        die "no prebuilt .deb for architecture '$ARCH' yet — build from source instead, see CONTRIBUTING.md"
    fi
    TMP="$(mktemp -d)"
    trap 'rm -rf "$TMP"' EXIT
    curl -fsSL -o "$TMP/anonveil.deb" "$URL"
    dpkg -i "$TMP/anonveil.deb" || apt-get install -f -y
    log "Installed. Run: sudo anonveil start"
    exit 0
fi

log "Unrecognized distro — falling back to a prebuilt binary."
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
    aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
    *) die "unsupported architecture: $ARCH" ;;
esac

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
BASE_URL="https://github.com/${REPO}/releases/latest/download"

curl -fsSL -o "$TMP/anonveil.tar.gz" "${BASE_URL}/anonveil-${TARGET}.tar.gz"
curl -fsSL -o "$TMP/SHA256SUMS" "${BASE_URL}/SHA256SUMS"

log "Verifying checksum..."
( cd "$TMP" && grep "anonveil-${TARGET}.tar.gz" SHA256SUMS | sha256sum -c - )

tar -xzf "$TMP/anonveil.tar.gz" -C "$TMP"
install -Dm755 "$TMP/anonveil" "${BIN_DIR}/anonveil"
mkdir -p /etc/anonveil /var/lib/anonveil /var/log/anonveil

log "Installed to ${BIN_DIR}/anonveil."
log "Make sure 'tor' and 'nftables' are installed, then run: sudo anonveil start"
