#!/bin/sh
# Regenerate docs/assets/{hero.gif,dashboard.png,help.png} for real —
# runs the actual compiled `anonveil` binary against a real `tor` daemon
# and real nftables, exactly like MANUAL_TESTING.md's release checklist,
# with VHS (https://github.com/charmbracelet/vhs) driving and recording
# the terminal. Not run in CI: it needs root, a real network, and a
# headless-Chromium-based screen recorder all inside one privileged
# container — a meaningful flake surface for something that only needs
# to run when the CLI's actual output changes, not on every push.
#
# Usage (from the repo root, Linux host or the same Docker pattern used
# throughout this project's own development):
#   sudo ./scripts/generate-screenshots.sh
#
# Or, matching how this was developed and verified — a disposable
# privileged container with no host-side setup at all:
#   docker run --rm --privileged --cap-add=NET_ADMIN \
#     -v "$PWD:/workspace" -w /workspace -e VHS_NO_SANDBOX=true \
#     rust:1-bookworm ./scripts/generate-screenshots.sh
set -eu

if [ "$(id -u)" -ne 0 ]; then
    echo "must run as root (needs to load real nftables rules and run a real tor daemon)" >&2
    exit 1
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

echo "== installing build/runtime/capture dependencies =="
apt-get update -qq
apt-get install -y -qq \
    curl ffmpeg fontconfig fonts-dejavu-core tor nftables sudo \
    libnss3 libatk1.0-0 libatk-bridge2.0-0 libcups2 libdrm2 libxkbcommon0 \
    libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libgbm1 libasound2 \
    libpango-1.0-0 libcairo2 libxrender1 libxext6 >/dev/null

if ! command -v ttyd >/dev/null 2>&1; then
    curl -fsSLo /usr/local/bin/ttyd \
        https://github.com/tsl0922/ttyd/releases/download/1.7.7/ttyd.x86_64
    chmod +x /usr/local/bin/ttyd
fi
if ! command -v vhs >/dev/null 2>&1; then
    curl -fsSLo /tmp/vhs.deb \
        https://github.com/charmbracelet/vhs/releases/download/v0.11.0/vhs_0.11.0_amd64.deb
    dpkg -i /tmp/vhs.deb >/dev/null 2>&1 || apt-get install -f -y -qq >/dev/null
fi

# CAPTURE-ONLY environment accommodation, not an AnonVeil behavior change:
# the containers this script is designed to run in (see the docker one-
# liner above) have no init system, so the real `systemctl` calls
# anonveil-priv::systemd.rs makes against tor.service/tor@default.service
# can't work here the way they do on a real desktop (or on the
# e2e-smoke-test CI job, which runs directly on a real systemd VM). This
# shim gives `systemctl` just enough of an interface for AnonVeil's own,
# completely unmodified code to succeed, by actually managing a real
# `tor` process directly underneath it.
# Always overwrite: one of the apt packages above may have pulled in a
# real (but non-functional here, see above) systemctl as a dependency.
cat > /usr/bin/systemctl <<'SHIM'
#!/bin/sh
case "$1 $2" in
    "list-unit-files tor.service"|"list-unit-files tor@default.service")
        echo "tor.service                   enabled"
        ;;
    "reload-or-restart tor.service"|"reload-or-restart tor@default.service")
        pkill -x tor 2>/dev/null || true
        sleep 1
        mkdir -p /run/tor && chown debian-tor:debian-tor /run/tor
        su -s /bin/sh debian-tor -c \
            "/usr/bin/tor -f /etc/tor/torrc --RunAsDaemon 1 --PidFile /run/tor/tor.pid"
        sleep 2
        ;;
    *)
        exit 0
        ;;
esac
SHIM
chmod +x /usr/bin/systemctl

# Another capture-only environment accommodation: Docker itself bind-
# mounts /etc/resolv.conf into the container, and AnonVeil unlinking/
# rewriting an active bind-mount target fails with EBUSY — a Docker
# quirk that doesn't exist on a real (or virtualized, non-containerized)
# Linux host, which is what AnonVeil actually targets. Replace it with a
# plain file holding the same content first.
cp /etc/resolv.conf /tmp/resolv.conf.orig
umount /etc/resolv.conf 2>/dev/null || true
cp /tmp/resolv.conf.orig /etc/resolv.conf

echo "== building anonveil =="
cargo build --release -p anonveil-cli
install -Dm755 target/release/anonveil /usr/local/bin/anonveil
mkdir -p /etc/anonveil
[ -f /etc/anonveil/config.toml ] || cp config/config.example.toml /etc/anonveil/config.toml

# Belt-and-braces: whatever state a previous run (or a previous failed
# run of this same script) left behind, start clean.
anonveil stop --force >/dev/null 2>&1 || true

export VHS_NO_SANDBOX=true
cd docs/vhs

echo "== recording help.png =="
vhs help.tape
rm -f help-tmp.gif

echo "== recording dashboard.png (runs a real start/check first) =="
vhs dashboard.tape
rm -f dashboard-tmp.gif
anonveil stop --force >/dev/null 2>&1 || true

echo "== recording hero.gif (full start -> check -> dashboard -> stop) =="
vhs hero.tape
anonveil stop --force >/dev/null 2>&1 || true

cd "$REPO_ROOT"
mkdir -p docs/assets
mv docs/vhs/help.png docs/vhs/dashboard.png docs/vhs/hero.gif docs/assets/

echo "== done =="
ls -la docs/assets/
