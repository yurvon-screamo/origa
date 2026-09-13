#!/bin/sh
# Custom entrypoint: start crond in background, then exec the main process (trail).
# tini runs as PID 1 (see Dockerfile ENTRYPOINT) and handles signal forwarding + zombie reaping.
CROND_DIR=/tmp/crond
mkdir -p "$CROND_DIR"
cp /app/crontab "$CROND_DIR/root"
crond -c "$CROND_DIR" -l 8

# GeoIP database must be present before `trail` starts: TrailBase loads it
# once at boot. Fail-open — a download failure must not block startup.
GEOIP_TIMEOUT=60 /app/geoip.sh || echo "[entrypoint] geoip: fetch failed, starting without geo"

exec "$@"
