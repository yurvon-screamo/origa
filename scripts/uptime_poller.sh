#!/usr/bin/env bash
# Origa uptime poller — the RF-side sentinel on the Aeza SPb VM.
#
# Runs every 15 minutes from cron (see docs/runbooks/uptime-monitoring.md):
#   * DNS assertions against pinned resolvers (RU pair + world),
#   * the pinned-IP HTTP probe of the RF branch of s3.origa.uwuwu.net,
#   * the TSPU reachability probe of the Railway-served hosts from RF,
#   * the openssl expiry probe of the umami certificate (no HTTP pings),
#   * a heartbeat ping to HetrixTools at the end of every completed cycle.
#
# Alert semantics (the anti-flap contract from the runbook):
#   * a check that fails twice in a row sends one ALERT (and one RESOLVE
#     when it comes back green);
#   * both RU resolvers being unreachable is a WARN, not an incident
#     (single operator behind anycast);
#   * the poller itself never alerts on its own dead egress — that case
#     is covered by the HetrixTools heartbeat going silent.
#
# Required environment (set in the cron environment on the VM):
#   TG_BOT_TOKEN, TG_CHAT_ID   — Telegram alert delivery
#   HETRIX_HEARTBEAT_URL       — HetrixTools heartbeat ping URL
# Optional:
#   POLLER_STATE_FILE          — anti-flap counters (default below)

set -euo pipefail

readonly STATE_FILE="${POLLER_STATE_FILE:-/var/tmp/uptime_poller.state}"
readonly RF_IP="193.233.217.243"
readonly RU_RESOLVER_PRIMARY="77.88.8.8"
readonly RU_RESOLVER_SECONDARY="77.88.8.1"
readonly WORLD_RESOLVER="1.1.1.1"
readonly UMAMI_CERT_HORIZON_DAYS=21

# Full hostnames (the landing is the bare domain — no second .origa label).
readonly ORIGA_HOST="origa.uwuwu.net"
readonly APP_HOST="app.origa.uwuwu.net"
readonly UMAMI_HOST="umami.origa.uwuwu.net"
readonly CONTENT_HOST="content.origa.uwuwu.net"
readonly S3_HOST="s3.origa.uwuwu.net"

readonly ORIGA_CNAME="c2qj368z.up.railway.app."
readonly APP_CNAME="9fmm6y4e.up.railway.app."
readonly UMAMI_CNAME="07wyrdxw.up.railway.app."
readonly CONTENT_CNAME="origa.t3.tigrisbucket.io."
readonly S3_WORLD_CNAME="d3gbi3wo8j4c2w.cloudfront.net."

log() { printf '%s %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$*"; }

send_telegram() {
  local text="$1"
  if [[ -z "${TG_BOT_TOKEN:-}" || -z "${TG_CHAT_ID:-}" ]]; then
    log "WARN telegram env missing, alert dropped: $text"
    return
  fi
  # A dropped notification must not kill the cycle.
  curl -fsS --max-time 15 \
    "https://api.telegram.org/bot${TG_BOT_TOKEN}/sendMessage" \
    --data-urlencode "chat_id=${TG_CHAT_ID}" \
    --data-urlencode "text=${text}" > /dev/null 2>&1 \
    || log "WARN telegram send failed"
}

# Anti-flap counter: bumps the check's counter, fires ALERT exactly on the
# second consecutive failure, RESOLVE on the first success after an alert.
record_outcome() {
  local check="$1" outcome="$2" context="$3"
  local key="fail:${check}"
  local previous
  # `|| true` keeps pipefail from killing the assignment when the state file
  # does not exist yet (first cycle after install).
  previous=$(awk -v k="$key" '$1 == k { print $2 }' "$STATE_FILE" 2>/dev/null | head -n1 || true)
  previous="${previous:-0}"

  if [[ "$outcome" == "ok" ]]; then
    if (( previous >= 2 )); then
      send_telegram "RESOLVE [origa-poller] ${check}: recovered. ${context}"
    fi
    update_state "$key" 0
    return
  fi

  local count=$((previous + 1))
  update_state "$key" "$count"
  if (( count == 2 )); then
    local severity="ALERT"
    [[ "$outcome" == "warn" ]] && severity="WARN"
    send_telegram "${severity} [origa-poller] ${check} failing twice in a row. ${context}"
  fi
}

update_state() {
  local key="$1" value="$2"
  touch "$STATE_FILE"
  # Replace the key's line in place, append it if absent — exactly one line
  # per key, otherwise the counter readback returns two numbers and the
  # arithmetic test crashes the next cycle.
  awk -v k="$key" -v v="$value" \
    '$1 == k { seen = 1; print k, v; next } { print } END { if (!seen) print k, v }' \
    "$STATE_FILE" > "${STATE_FILE}.next"
  mv "${STATE_FILE}.next" "$STATE_FILE"
}

# --- DNS assertions ---------------------------------------------------------

# dig emits the CNAME chain; the first line carries the canonical target of
# the queried name itself.
cname_matches() {
  local resolver="$1" host="$2" expected="$3"
  local answer
  answer=$(dig +short "$host" "@$resolver" CNAME 2>/dev/null | head -n1 || true)
  [[ "$answer" == "$expected" ]]
}

s3_ru_is_vm_ip() {
  local resolver="$1"
  local answer
  answer=$(dig +short "$S3_HOST" "@$resolver" A 2>/dev/null | head -n1 || true)
  [[ "$answer" == "$RF_IP" ]]
}

# The geo split survives only on s3: both RU resolvers must answer with the
# VM's A record, and the remaining names with their Railway/Tigris CNAMEs —
# asserted through both resolvers, so a dead secondary cannot drift silently.
check_dns_ru() {
  local failures=0 detail="" resolver host expected

  for resolver in "$RU_RESOLVER_PRIMARY" "$RU_RESOLVER_SECONDARY"; do
    if ! s3_ru_is_vm_ip "$resolver"; then
      failures=$((failures + 1))
      detail+="s3@${resolver}!=RF-IP; "
    fi
    for host in "$ORIGA_HOST" "$APP_HOST" "$UMAMI_HOST" "$CONTENT_HOST"; do
      case "$host" in
        "$ORIGA_HOST") expected="$ORIGA_CNAME" ;;
        "$APP_HOST") expected="$APP_CNAME" ;;
        "$UMAMI_HOST") expected="$UMAMI_CNAME" ;;
        "$CONTENT_HOST") expected="$CONTENT_CNAME" ;;
      esac
      if ! cname_matches "$resolver" "$host" "$expected"; then
        failures=$((failures + 1))
        detail+="${host}@${resolver}!=${expected}; "
      fi
    done
  done

  if (( failures == 0 )); then
    record_outcome "dns-ru" ok "all assertions matched"
  else
    record_outcome "dns-ru" fail "$detail"
  fi
}

check_dns_world() {
  local failures=0 detail="" host expected

  if ! cname_matches "$WORLD_RESOLVER" "$S3_HOST" "$S3_WORLD_CNAME"; then
    failures=$((failures + 1))
    detail+="s3@world!=${S3_WORLD_CNAME}; "
  fi

  for host in "$ORIGA_HOST" "$APP_HOST" "$UMAMI_HOST" "$CONTENT_HOST"; do
    case "$host" in
      "$ORIGA_HOST") expected="$ORIGA_CNAME" ;;
      "$APP_HOST") expected="$APP_CNAME" ;;
      "$UMAMI_HOST") expected="$UMAMI_CNAME" ;;
      "$CONTENT_HOST") expected="$CONTENT_CNAME" ;;
    esac
    if ! cname_matches "$WORLD_RESOLVER" "$host" "$expected"; then
      failures=$((failures + 1))
      detail+="${host}@world!=${expected}; "
    fi
  done

  if (( failures == 0 )); then
    record_outcome "dns-world" ok "all assertions matched"
  else
    record_outcome "dns-world" fail "$detail"
  fi
}

# Both RU resolvers dead at once is a single-operator (Yandex anycast) outage:
# warn, never incident — the world side still works.
check_ru_resolvers_alive() {
  local alive=0 resolver
  for resolver in "$RU_RESOLVER_PRIMARY" "$RU_RESOLVER_SECONDARY"; do
    dig +short uwuwu.net "@$resolver" > /dev/null 2>&1 && alive=$((alive + 1))
  done
  if (( alive == 2 )); then
    record_outcome "ru-resolvers" ok "both alive"
  else
    record_outcome "ru-resolvers" warn "only ${alive} of 2 alive"
  fi
}

# --- HTTP probes ------------------------------------------------------------

# Pinned-IP probe of the RF branch: TLS+SNI through the VM's nginx. The
# CloudFront misroute guard (the kz1 lesson) has two legs: the VM's nginx
# signs responses with a versioned "server: nginx/x.y.z" while the world
# branch (CloudFront in front of Tigris) serves a bare "server: nginx"; and
# CloudFront always adds via/x-cache/x-amz-cf-* markers that the VM's direct
# path never carries. The Tigris x-tigris-* headers are present on BOTH
# branches (the Tigris origin sets them) — they do not discriminate.
check_rf_branch() {
  local headers
  headers=$(curl -sS -D - -o /dev/null --max-time 20 \
    --resolve "${S3_HOST}:443:${RF_IP}" \
    "https://${S3_HOST}/manifest.json" 2>/dev/null) || headers=""
  local http_code
  http_code=$(head -n1 <<< "$headers" | awk '{print $2}')
  http_code="${http_code:-000}"

  if [[ "$http_code" != "200" ]]; then
    record_outcome "rf-branch" fail "http_code=${http_code}"
    return
  fi
  if ! grep -qi '^server: nginx/' <<< "$headers"; then
    record_outcome "rf-branch" fail "no VM nginx header"
    return
  fi
  if grep -qiE '^(via|x-cache|x-amz-cf-id):' <<< "$headers"; then
    record_outcome "rf-branch" fail "CloudFront markers on the pinned IP"
    return
  fi
  record_outcome "rf-branch" ok "200 from VM nginx"
}

# Direct Railway reachability from an RF datacenter vantage: catches the full
# TSPU block of the Railway edge (ADR-049 class) and the silent custom-domain
# slap — a 4xx here is the RF side of the Railway fallback, never green.
# Partial residential degradation stays a blind spot.
check_rf_railway() {
  local host="$1" http_code
  http_code=$(curl -sS -o /dev/null -w '%{http_code}' --max-time 20 \
    "https://${host}/" 2>/dev/null) || http_code="000"
  case "$http_code" in
    2*|3*)
      record_outcome "rf-railway-${host}" ok "http_code=${http_code}"
      ;;
    *)
      record_outcome "rf-railway-${host}" fail "http_code=${http_code}"
      ;;
  esac
}

# --- Certificate probe ------------------------------------------------------

# umami certificate expiry without a single HTTP ping: openssl talks TLS
# directly, so the zero-scale contract for the umami service holds.
check_umami_certificate() {
  local enddate days_left now then_ts
  enddate=$(echo | openssl s_client -servername "$UMAMI_HOST" \
    -connect "$UMAMI_HOST:443" 2>/dev/null \
    | openssl x509 -noout -enddate 2>/dev/null | cut -d= -f2 || true)
  if [[ -z "$enddate" ]]; then
    record_outcome "umami-cert" fail "no certificate retrieved"
    return
  fi
  now=$(date +%s)
  then_ts=$(date -d "$enddate" +%s)
  days_left=$(((then_ts - now) / 86400))
  if (( days_left < UMAMI_CERT_HORIZON_DAYS )); then
    record_outcome "umami-cert" warn "expires in ${days_left} days"
  else
    record_outcome "umami-cert" ok "${days_left} days left"
  fi
}

# --- Cycle ------------------------------------------------------------------

send_heartbeat() {
  # The heartbeat proves the poller itself is alive, so it fires at the end
  # of every completed cycle regardless of individual check outcomes.
  if [[ -n "${HETRIX_HEARTBEAT_URL:-}" ]]; then
    # -L: some intermediate answers redirect; a heartbeat ping must follow.
    curl -fsSL --max-time 15 "$HETRIX_HEARTBEAT_URL" > /dev/null 2>&1 \
      || log "WARN heartbeat ping failed"
  else
    log "WARN HETRIX_HEARTBEAT_URL not set"
  fi
}

main() {
  log "cycle start"
  check_dns_ru
  check_dns_world
  check_ru_resolvers_alive
  check_rf_branch
  check_rf_railway "$ORIGA_HOST"
  check_rf_railway "$APP_HOST"
  check_umami_certificate
  send_heartbeat
  log "cycle end"
}

main "$@"
