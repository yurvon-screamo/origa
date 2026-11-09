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
GEOIP_URL="${GEOIP_URL:-https://github.com/P3TERX/GeoLite.mmdb/releases/latest/download/GeoLite2-Country.mmdb}"
FRESHNESS_DAYS="${GEOIP_FRESHNESS_DAYS:-6}"
CURL_TIMEOUT="${GEOIP_TIMEOUT:-120}"

# mmdb format validation instead of a size threshold: a size gate guesses
# the database's internals and drifts away from reality (the real
# GeoLite2-Country is ~3.2 MB, an old "~8 MB" threshold rejected every
# valid weekly refresh). The MaxMind-DB spec puts the metadata section at
# the end of the file, searchable by the marker \xAB\xCD\xEF + "MaxMind.com",
# and caps its size at 128 KiB — so a truncated download always loses the
# marker, while a valid database always has it, at any size.
MMDB_MARKER=$(printf '\253\315\357MaxMind.com')
MMDB_TAIL_BYTES=131072 # 128 KiB, max metadata section size per the spec

log() {
	printf '[geoip %s] %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$*"
}

# A truncated or corrupt file loses the end-of-file metadata marker.
is_valid_mmdb() {
	tail -c "${MMDB_TAIL_BYTES}" "$1" | LC_ALL=C grep -qF "${MMDB_MARKER}"
}

# --- Skip if fresh enough (a corrupt or truncated leftover fails the marker
# check and gets re-downloaded instead of being trusted) ---
if [ -f "${GEOIP_DEST}" ] && is_valid_mmdb "${GEOIP_DEST}"; then
	file_age=$(($(date +%s) - $(stat -c %Y "${GEOIP_DEST}")))
	if [ "${file_age}" -lt $((FRESHNESS_DAYS * 86400)) ]; then
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

if ! is_valid_mmdb "${tmp_file}"; then
	log "No mmdb metadata in the tail (invalid or truncated download) — discarding."
	exit 1
fi

mv "${tmp_file}" "${GEOIP_DEST}"
log "Installed ${GEOIP_DEST} ($(stat -c %s "${GEOIP_DEST}") bytes)."
