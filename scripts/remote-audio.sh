#!/usr/bin/env bash
#
# remote-audio.sh — run oxeye with its speech routed to a remote PulseAudio/PipeWire server.
#
# Set PULSE_SERVER to where the audio should go (e.g. a reverse-forwarded socket
# `tcp:localhost:24713`, or a tailnet host `tcp:station:4713`), then run this. It restarts
# speech-dispatcher so it picks up PULSE_SERVER (oxeye only respawns the daemon if none is
# running), then launches oxeye in speech mode. See docs/remote-audio.md for the full setup.
#
# Usage:
#   PULSE_SERVER=tcp:localhost:24713 scripts/remote-audio.sh [extra cargo-run args...]
set -euo pipefail

: "${PULSE_SERVER:?Set PULSE_SERVER to your remote audio server — see docs/remote-audio.md}"
export PULSE_SERVER

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Drop any stale local speech-dispatcher so oxeye respawns it with PULSE_SERVER inherited.
pkill -u "$(id -u)" speech-dispatcher 2>/dev/null || true

echo "routing oxeye speech to PULSE_SERVER=$PULSE_SERVER" >&2
exec cargo run -q --manifest-path "$repo_root/Cargo.toml" -p oxeye-linux -- "$@"
