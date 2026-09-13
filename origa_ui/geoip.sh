#!/bin/sh
# GeoIP database provisioning for TrailBase request-log geo enrichment.
#
# TrailBase loads <traildepot>/GeoLite2-Country.mmdb once at startup and uses
# it to resolve client_ip -> country in the admin logs UI (computed at query
# time, so a freshly installed database also covers historical entries).
#
# Two invocation points:
#   * entrypoint.sh runs it before `trail run` (covers fresh volumes)
#   * crond runs it weekly (picked up on the next container restart)
#
# Data source: P3TERX/GeoLite.mmdb republishes the MaxMind GeoLite2-Country
# database weekly as plain mmdb, no MaxMind account required (direct MaxMind
# signup is geo-blocked for us). MaxMind attribution stays inside the mmdb
# metadata; see https://github.com/P3TERX/GeoLite.mmdb
#
# Fail-open by design: a failed download never blocks container startup —
# TrailBase runs without geo info until the next successful fetch.
set -eu

# --- Configuration ---
GEOIP_DEST="${TRAILBASE_DATA_DIR:-/app/traildepot}/GeoLite2-Country.mmdb"
GEOIP_URL="https://github.com/P3TERX/GeoLite.mmdb/releases/latest/download/GeoLite2-Country.mmdb"
FRESHNESS_DAYS="${GEOIP_FRESHNESS_DAYS:-6}"
CURL_TIMEOUT="${GEOIP_TIMEOUT:-120}"
MIN_SIZE_BYTES=4194304 # valid builds are ~8 MB; a truncated download is smaller

log() {
    printf '[geoip %s] %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$*"
}

# --- Skip if fresh enough (a corrupt or truncated leftover fails the size check
# and gets re-downloaded instead of being trusted) ---
if [ -f "${GEOIP_DEST}" ]; then
    existing_size=$(stat -c %s "${GEOIP_DEST}")
    file_age=$(( $(date +%s) - $(stat -c %Y "${GEOIP_DEST}") ))
    if [ "${existing_size}" -ge "${MIN_SIZE_BYTES}" ] \
        && [ "${file_age}" -lt $(( FRESHNESS_DAYS * 86400 )) ]; then
        log "Database is fresh (${FRESHNESS_DAYS}d window), skipping."
        exit 0
    fi
fi

mkdir -p "$(dirname "${GEOIP_DEST}")"

# --- Atomic download: tmp file in the target directory, then rename ---
log "Downloading GeoLite2-Country..."
rm -f "${GEOIP_DEST}".tmp.* # orphans from a SIGKILLed run; -f tolerates no matches
tmp_file="${GEOIP_DEST}.tmp.$$"
trap 'rm -f "${tmp_file}"' EXIT

curl -sSfL --max-time "${CURL_TIMEOUT}" --retry 3 --retry-delay 2 \
    -o "${tmp_file}" "${GEOIP_URL}"

downloaded_size=$(stat -c %s "${tmp_file}")
if [ "${downloaded_size}" -lt "${MIN_SIZE_BYTES}" ]; then
    log "Downloaded file too small (${downloaded_size} bytes) — discarding."
    exit 1
fi

mv "${tmp_file}" "${GEOIP_DEST}"
log "Installed ${GEOIP_DEST} (${downloaded_size} bytes)."
