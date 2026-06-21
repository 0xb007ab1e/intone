#!/usr/bin/env bash
#
# test-readout.sh — headless/remote functional check for the oxeye screen reader.
#
# Runs oxeye ONCE in TEXT mode (OXEYE_SPEECH=text: prints announcements, no audio, no
# daemon), then for each test app: launches it so a focus event fires, captures what oxeye
# announces (plus any WARN diagnostics), and tears that app down before the next one. On
# exit a trap kills everything — no idle processes left behind. Safe to run repeatedly.
#
# Requires a graphical KDE/Wayland session (for the a11y bus + KeyboardMonitor).
#
# Usage:
#   scripts/test-readout.sh                 # default apps: kdialog --msgbox, then kcalc
#   scripts/test-readout.sh kcalc           # just one app
#   scripts/test-readout.sh kdialog --calendar
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bin="$repo_root/target/debug/oxeye-linux"

if [ "$#" -gt 0 ]; then
  app_specs=("$*")
else
  app_specs=("kdialog --msgbox oxeye-readout-test" "kcalc")
fi

log="$(mktemp -t oxeye-readout.XXXXXX)"
pids=()

cleanup() {
  set +e
  local pid
  for pid in "${pids[@]:-}"; do
    [ -n "${pid:-}" ] && kill -TERM "$pid" 2>/dev/null
  done
  sleep 0.3
  for pid in "${pids[@]:-}"; do
    [ -n "${pid:-}" ] && kill -KILL "$pid" 2>/dev/null
  done
  rm -f "$log"
}
trap cleanup EXIT INT TERM

echo "building oxeye-linux…" >&2
( cd "$repo_root" && cargo build -q -p oxeye-linux )

# Clear any stray oxeye from a prior run so the Orca D-Bus name is free.
pkill -f "$bin" 2>/dev/null || true

echo "starting oxeye (text mode; no audio, no daemon)…" >&2
OXEYE_SPEECH=text "$bin" >"$log" 2>&1 &
pids+=("$!")

# Wait until oxeye is in its event loop (bounded; ~10s max).
for _ in $(seq 1 100); do
  grep -q "spike running" "$log" && break
  sleep 0.1
done

for spec in "${app_specs[@]}"; do
  read -ra cmd <<< "$spec"
  echo
  echo "=== reading app: ${cmd[*]} ==="
  mark="$(wc -l < "$log")"
  "${cmd[@]}" >/dev/null 2>&1 &
  app_pid="$!"
  pids+=("$app_pid")
  sleep 2.5
  if ! tail -n +"$((mark + 1))" "$log" | grep -aE '^\[(say|silence)\]|WARN|ERROR|failed'; then
    echo "(nothing captured)"
  fi
  # Tear this app down before the next one — no idle leftovers.
  kill -TERM "$app_pid" 2>/dev/null || true
  sleep 0.3
  kill -KILL "$app_pid" 2>/dev/null || true
done

echo
echo "=== done — all spawned processes torn down on exit ==="
