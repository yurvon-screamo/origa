#!/bin/sh
set -e

cd /app/origa_landing
ORIGA_LANDING_BASE_URL=${ORIGA_LANDING_BASE_URL:-https://origa.app} \
    /app/origa_landing &

cd /app

caddy run --config /app/Caddyfile &

exec /app/trail run --address 127.0.0.1:8080 --public-dir /app/public --spa
