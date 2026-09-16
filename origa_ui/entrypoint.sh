#!/bin/sh
# Custom entrypoint: start crond in background, then exec the main process (trail).
# tini runs as PID 1 (see Dockerfile ENTRYPOINT) and handles signal forwarding + zombie reaping.
CROND_DIR=/tmp/crond
mkdir -p "$CROND_DIR"
cp /app/crontab "$CROND_DIR/root"
crond -c "$CROND_DIR" -l 8

# GeoIP database must be present before `trail` starts: TrailBase loads it
# once at boot. Fail-open — a download failure must not block startup. The
# 45s budget keeps a worst-case fetch inside the healthcheck window
# (5s start-period + 3 retries × 30s) together with a cold trail start.
GEOIP_TIMEOUT=45 /app/geoip.sh || echo "[entrypoint] geoip: fetch failed, starting without geo"

# The data volume shadows /app/traildepot, so the auth-UI wasm baked into the
# image would go stale across trail upgrades; re-sync it from /opt on every
# boot (best-effort — the previously deployed component keeps working).
mkdir -p /app/traildepot/wasm
cp -f /opt/trailbase/wasm/trailbase_auth_ui_component.wasm \
  /app/traildepot/wasm/ 2>/dev/null || echo "[entrypoint] wasm auth-ui sync failed"

# THP_DISABLE=1 (also true/yes): launch trail through the static THP-disable
# launcher. The prctl flag survives execve and stops the host's THP=always
# policy from inflating the cgroup memory accounting of trail's mimalloc
# arenas. Opt-in + fail-open: without the variable (or the binary) behavior
# is unchanged from the plain exec below.
case "${THP_DISABLE:-0}" in
1 | true | yes)
    if [ -x /app/thp_off ]; then
        exec /app/thp_off "$@"
    fi
    echo "[entrypoint] thp_off binary missing, starting without THP disable"
    ;;
esac

exec "$@"
