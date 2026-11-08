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
  previous=$(awk -v k="$key" '$1 == k { print $2 }' "$STATE_FILE" 2>/dev/null || echo 0)
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
  awk -v k="$key" -v v="$value" '$1 == k { skip = 1 } { if (!skip) print } END { if (!skip) print k, v }' \
    "$STATE_FILE" > "${STATE_FILE}.next"
  printf '%s %s\n' "$key" "$value" >> "${STATE_FILE}.next"
  mv "${STATE_FILE}.next" "$STATE_FILE"
}

# --- DNS assertions ---------------------------------------------------------

# dig emits the CNAME chain; the first line carries the canonical target of
# the queried name itself.
cname_matches() {
  local resolver="$1" name="$2" expected="$3"
  local answer
  answer=$(dig +short "$name.origa.uwuwu.net" "@$resolver" CNAME 2>/dev/null | head -n1)
  [[ "$answer" == "$expected" ]]
}

s3_ru_is_vm_ip() {
  local resolver="$1"
  local answer
  answer=$(dig +short s3.origa.uwuwu.net "@$resolver" A 2>/dev/null | head -n1)
  [[ "$answer" == "$RF_IP" ]]
}

# The geo split survives only on s3: both RU resolvers must answer with the
# VM's A record, and the remaining names with their Railway/Tigris CNAMEs.
check_dns_ru() {
  local failures=0 detail=""

  for resolver in "$RU_RESOLVER_PRIMARY" "$RU_RESOLVER_SECONDARY"; do
    if ! s3_ru_is_vm_ip "$resolver"; then
      failures=$((failures + 1))
      detail+="s3@${resolver}!=RF-IP; "
    fi
  done

  local name
  for name in origa app umami content; do
    local expected
    case "$name" in
      origa) expected="$ORIGA_CNAME" ;;
      app) expected="$APP_CNAME" ;;
      umami) expected="$UMAMI_CNAME" ;;
      content) expected="$CONTENT_CNAME" ;;
    esac
    if ! cname_matches "$RU_RESOLVER_PRIMARY" "$name" "$expected"; then
      failures=$((failures + 1))
      detail+="${name}@RU!=${expected}; "
    fi
  done

  if (( failures == 0 )); then
    record_outcome "dns-ru" ok "all assertions matched"
  else
    record_outcome "dns-ru" fail "$detail"
  fi
}

check_dns_world() {
  local failures=0 detail=""

  if ! cname_matches "$WORLD_RESOLVER" s3 "$S3_WORLD_CNAME"; then
    failures=$((failures + 1))
    detail+="s3@world!=${S3_WORLD_CNAME}; "
  fi

  local name
  for name in origa app umami content; do
    local expected
    case "$name" in
      origa) expected="$ORIGA_CNAME" ;;
      app) expected="$APP_CNAME" ;;
      umami) expected="$UMAMI_CNAME" ;;
      content) expected="$CONTENT_CNAME" ;;
    esac
    if ! cname_matches "$WORLD_RESOLVER" "$name" "$expected"; then
      failures=$((failures + 1))
      detail+="${name}@world!=${expected}; "
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
# CloudFront misroute guard (the kz1 lesson) asserts the response carries the
# VM's nginx header and not the Tigris-served-from marker of the world branch.
check_rf_branch() {
  local headers
  headers=$(curl -sS -D - -o /dev/null --max-time 20 \
    --resolve "s3.origa.uwuwu.net:443:${RF_IP}" \
    "https://s3.origa.uwuwu.net/manifest.json" 2>/dev/null) || headers=""
  local http_code
  http_code=$(head -n1 <<< "$headers" | awk '{print $2}')
  http_code="${http_code:-000}"

  if [[ "$http_code" != "200" ]]; then
    record_outcome "rf-branch" fail "http_code=${http_code}"
    return
  fi
  if grep -qi '^x-tigris-served-from:' <<< "$headers"; then
    record_outcome "rf-branch" fail "world branch leaked into the pinned IP"
    return
  fi
  if ! grep -qi '^server: nginx/' <<< "$headers"; then
    record_outcome "rf-branch" fail "no VM nginx header"
    return
  fi
  record_outcome "rf-branch" ok "200 from VM nginx"
}

# Direct Railway reachability from an RF datacenter vantage: catches the full
# TSPU block of the Railway edge (ADR-049 class). Partial residential
# degradation stays a blind spot.
check_rf_railway() {
  local host="$1" http_code
  http_code=$(curl -sS -o /dev/null -w '%{http_code}' --max-time 20 \
    "https://${host}.origa.uwuwu.net/" 2>/dev/null) || http_code="000"
  if [[ "$http_code" == "000" || "$http_code" == 5* ]]; then
    record_outcome "rf-railway-${host}" fail "http_code=${http_code}"
  else
    record_outcome "rf-railway-${host}" ok "http_code=${http_code}"
  fi
}

# --- Certificate probe ------------------------------------------------------

# umami certificate expiry without a single HTTP ping: openssl talks TLS
# directly, so the zero-scale contract for the umami service holds.
check_umami_certificate() {
  local enddate days_left now then_ts
  enddate=$(echo | openssl s_client -servername umami.origa.uwuwu.net \
    -connect umami.origa.uwuwu.net:443 2>/dev/null \
    | openssl x509 -noout -enddate 2>/dev/null | cut -d= -f2)
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
    curl -fsS --max-time 15 "$HETRIX_HEARTBEAT_URL" > /dev/null 2>&1 \
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
  check_rf_railway origa
  check_rf_railway app
  check_umami_certificate
  send_heartbeat
  log "cycle end"
}

main "$@"
