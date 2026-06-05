#!/bin/sh
set -e

cd /app/origa_landing
echo "Starting origa_landing..."
ORIGA_LANDING_BASE_URL=${ORIGA_LANDING_BASE_URL:-https://origa.app} \
    /usr/local/bin/origa_landing 2>&1 &
LANDING_PID=$!
cd /app

caddy run --config /app/Caddyfile &

# Wait a moment and check if landing is still alive
sleep 2
if ! kill -0 "$LANDING_PID" 2>/dev/null; then
    echo "origa_landing CRASHED (PID $LANDING_PID)"
    wait "$LANDING_PID" 2>/dev/null
    echo "Exit code: $?"
fi

exec /app/trail run --address 127.0.0.1:8080 --public-dir /app/public --spa
