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

# The image no longer ships the auth-UI WASM component (memory: wasmtime
# engine + per-boot JIT cost ~250 MB). The data volume, however, may still
# hold one from an older image — remove it so trail starts without the
# component (fail-safe: nothing to remove on a fresh volume).
rm -f /app/traildepot/wasm/*.wasm

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
