#!/usr/bin/env bash
set -euo pipefail
# Synthetic OS keyring acceptance; never reuse the caller's desktop bus.
task_root=$(cd -- "$(dirname -- "$0")/../.." && pwd)
if [[ ${ZIX_ISOLATED_KEYRING:-} != 1 ]]; then
  task_private=$(mktemp -d)
  trap 'rm -rf -- "$task_private"' EXIT
  chmod 700 "$task_private"
  mkdir -m 700 "$task_private/data" "$task_private/config" "$task_private/runtime"
  env -u DBUS_SESSION_BUS_ADDRESS -u GNOME_KEYRING_CONTROL ZIX_ISOLATED_KEYRING=1 \
    XDG_DATA_HOME="$task_private/data" XDG_CONFIG_HOME="$task_private/config" \
    XDG_RUNTIME_DIR="$task_private/runtime" dbus-run-session -- bash "$0"
  exit
fi
cd "$task_root"
gnome-keyring-daemon --foreground --unlock --components=secrets \
  --control-directory="$XDG_RUNTIME_DIR/keyring" \
  < <(printf '%s' 'synthetic-test-password') >"$XDG_RUNTIME_DIR/provider.log" 2>&1 &
task_daemon=$!
trap 'kill "$task_daemon" 2>/dev/null || true; wait "$task_daemon" 2>/dev/null || true' EXIT
gdbus wait --session --timeout 5 org.freedesktop.secrets
task_ready=0
for ((task_attempt=0; task_attempt<50; task_attempt++)); do
  if gdbus call --session --dest org.freedesktop.secrets \
    --object-path /org/freedesktop/secrets/collection/login \
    --method org.freedesktop.DBus.Properties.Get org.freedesktop.Secret.Collection Locked >/dev/null 2>&1; then
    gdbus call --session --dest org.freedesktop.secrets --object-path /org/freedesktop/secrets \
      --method org.freedesktop.Secret.Service.SetAlias default /org/freedesktop/secrets/collection/login >/dev/null
    task_ready=1; break
  fi
  sleep 0.1
done
[[ $task_ready == 1 ]] || { echo 'Isolated provider unavailable; no acceptance claimed.' >&2; exit 2; }
cargo +1.97 test --locked --features os-keyring --test keyring -- --include-ignored
