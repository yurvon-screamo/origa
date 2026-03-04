#!/bin/sh
set -e

caddy run --config /app/Caddyfile &
trailbase_pid=$!

exec /app/trail run --address 127.0.0.1:8080 --public-dir /app/public --spa
