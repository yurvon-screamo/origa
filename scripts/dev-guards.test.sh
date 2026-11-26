#!/bin/sh
# Decision-table regression for scripts/dev-guards.sh (plan review C7 of
# uwuwu#1472): pure verdicts only — the ss//proc probes are exercised by the
# live runs in the task report. Plain asserts, no framework.
#
# Run: scripts/dev-guards.test.sh   (from anywhere)

set -u

# shellcheck source=scripts/dev-guards.sh disable=SC1091 # follow-mode off in the gate; the path is relative to this file
. "$(dirname -- "$0")/dev-guards.sh"

_pass=0
_fail=0

ok() { # ok <label> <command...> — the command must exit 0
	_t_label=$1
	shift
	if "$@" >/dev/null 2>&1; then
		_pass=$((_pass + 1))
	else
		_fail=$((_fail + 1))
		printf 'FAIL %s (expected ok, got refusal)\n' "$_t_label"
	fi
}

ko() { # ko <label> <command...> — the command must exit non-zero
	_t_label=$1
	shift
	if "$@" >/dev/null 2>&1; then
		_fail=$((_fail + 1))
		printf 'FAIL %s (expected refusal, got ok)\n' "$_t_label"
	else
		_pass=$((_pass + 1))
	fi
}

eq() { # eq <label> <expected> <actual>
	if [ "$2" = "$3" ]; then
		_pass=$((_pass + 1))
	else
		_fail=$((_fail + 1))
		printf 'FAIL %s (expected [%s], got [%s])\n' "$1" "$2" "$3"
	fi
}

# ---- TRAILBASE_URL: strict loopback contract ----
ok "tb loopback localhost" trailbase_is_local "http://localhost:4000"
ok "tb loopback 127.0.0.1" trailbase_is_local "http://127.0.0.1:4000"
ok "tb loopback ipv6 bracket" trailbase_is_local "http://[::1]:4000"
ok "tb loopback ipv6 bracket + path" trailbase_is_local "http://[::1]:4000/"
ok "tb loopback no port" trailbase_is_local "http://localhost"
ok "tb loopback 127.0.0.1 no port" trailbase_is_local "http://127.0.0.1"
ok "tb loopback trailing slash" trailbase_is_local "http://127.0.0.1:4000/"
ok "tb loopback https scheme" trailbase_is_local "https://localhost:4000"
ko "tb empty" trailbase_is_local ""
ko "tb prod host (the incident value)" trailbase_is_local "https://app.origa.uwuwu.net"
ko "tb prod cdn host" trailbase_is_local "https://s3.origa.uwuwu.net"
ko "tb lookalike subdomain" trailbase_is_local "http://localhost.evil.example"
ko "tb lan ip" trailbase_is_local "http://192.168.1.10:4000"
# review round 1 (common 2): brackets OUTSIDE the authority are data, not
# the host — the verdict must stay "not local" for these
ko "tb prod + bracketed ipv6 in query" trailbase_is_local "https://app.origa.uwuwu.net/path?[::1]"
ko "tb bracketed ipv6 in fragment" trailbase_is_local "/#[::1]"
ko "tb bracketed localhost in path" trailbase_is_local "/[localhost]"
ko "tb bracketed loopback ip in path" trailbase_is_local "/x[127.0.0.1]"
# review round 2: userinfo — the host is what follows the @, a loopback
# userinfo must not flip the verdict
ko "tb userinfo on prod (loopback creds)" trailbase_is_local "https://127.0.0.1:1@app.origa.uwuwu.net"
ko "tb userinfo loopback@loopback" trailbase_is_local "http://localhost@localhost:4000"
# review round 2 (low 1): absolute URL required — the client concatenates
# the base with API paths, so a schemeless value yields broken fetches
ko "tb schemeless (localhost:4000)" trailbase_is_local "localhost:4000"
ko "tb bare ipv6 authority" trailbase_is_local "::1"
ko "tb unbracketed ipv6 with port" trailbase_is_local "http://::1:4000"
ko "tb empty port" trailbase_is_local "http://localhost:"
ko "tb non-numeric port" trailbase_is_local "http://localhost:abc"
ko "tb port with garbage tail" trailbase_is_local "http://localhost:1:2"

# ---- desktop dev value (uwuwu#1663): unset/empty -> local backend ----
eq "td unset injects loopback" "http://127.0.0.1:4000" "$(tauri_trailbase_value)"
eq "td empty injects loopback (ADR-024)" "http://127.0.0.1:4000" "$(tauri_trailbase_value "")"
eq "td explicit loopback passes" "http://localhost:4000" "$(tauri_trailbase_value "http://localhost:4000")"
ko "td explicit prod refuses" tauri_trailbase_value "https://app.origa.uwuwu.net"
ko "td empty-scheme value refuses" tauri_trailbase_value "localhost:4000"

# ---- landing base: L1 pair contract ----
ok "landing app base alone" landing_base_present "https://app.origa.uwuwu.net" "" ""
ok "landing pair uri+prefix" landing_base_present "" "origa.uwuwu.net" "app"
ok "landing app base and pair both" landing_base_present "https://app" "o" "a"
ko "landing nothing" landing_base_present "" "" ""
ko "landing half pair (prefix missing)" landing_base_present "" "origa.uwuwu.net" ""
ko "landing half pair (uri missing)" landing_base_present "" "" "app"

# ---- listener verdict table (C3: serve-ui refuses ANY listener; e2e per env) ----
ko "serve-ui app own" listener_verdict serve-ui app own local
ko "serve-ui app stranger" listener_verdict serve-ui app stranger "-"
ko "serve-ui app unknown" listener_verdict serve-ui app unknown "-"
ko "e2e app stranger" listener_verdict e2e app stranger "-"
ok "e2e app own local (verified reuse)" listener_verdict e2e app own local
ko "e2e app own prod bundle" listener_verdict e2e app own notlocal
ko "e2e app own unverified env" listener_verdict e2e app own unverified
ok "e2e trailbase own (harness kills stale)" listener_verdict e2e trailbase own "-"
ok "e2e trailbase unknown" listener_verdict e2e trailbase unknown "-"
ko "e2e trailbase stranger" listener_verdict e2e trailbase stranger "-"
ok "e2e cdn own (read-only corpus)" listener_verdict e2e cdn own "-"
ko "e2e cdn stranger" listener_verdict e2e cdn stranger "-"
ok "e2e cdn unknown" listener_verdict e2e cdn unknown "-"

# GUARD_WHY carries a reason on refusals (NFT: remedy-bearing messages)
listener_verdict e2e app own notlocal
eq "refusal sets GUARD_WHY" "свой сервер собран с непроверенным или прод TrailBase" "$GUARD_WHY"

# ---- corpus serve chain (L2: exact `??` parity with resolveCdnSource —
# a SET-EMPTY var does not fall through; first SET var wins, then existence).
# Cases run sequentially at the top level: env is set per case, corpus_dir
# is called directly (its $() is plain command substitution — no env edits
# inside subshells).
_tmp=$(mktemp -d)
mkdir -p "$_tmp/seed" "$_tmp/dir" "$_tmp/home/origa-cdn" "$_tmp/nohome"
_old_home=$HOME

export ORIGA_CDN_CACHE_DIR="$_tmp/seed"
ok "corpus cache-dir wins (exit 0 as predicate)" corpus_dir
eq "corpus cache-dir prints" "$_tmp/seed" "$(corpus_dir)"

export ORIGA_CDN_CACHE_DIR="$_tmp/nope" ORIGA_CDN_DIR="$_tmp/dir"
ko "corpus set-but-missing: no fallback, refuses (parity with ??)" corpus_dir
eq "corpus set-but-missing prints nothing" "" "$(corpus_dir)"

unset ORIGA_CDN_CACHE_DIR
export ORIGA_CDN_DIR="$_tmp/dir"
ok "corpus dir fallback when cache unset" corpus_dir

unset ORIGA_CDN_DIR
HOME="$_tmp/home"
export HOME
ok "corpus home seed" corpus_dir

HOME="$_tmp/nohome"
export HOME
ko "corpus absent refuses (exit 1 as predicate)" corpus_dir
eq "corpus absent prints nothing" "" "$(corpus_dir)"

HOME=$_old_home
export HOME
unset ORIGA_CDN_CACHE_DIR ORIGA_CDN_DIR
rm -rf "$_tmp"

# creds resolve: shell env or .env (harness contract, C4)
_tmp2=$(mktemp -d)
eq "creds from shell" "0" "$(
	ORIGA_ADMIN_PASSWORD=x
	export ORIGA_ADMIN_PASSWORD
	creds_resolve "$_tmp2"
	echo $?
)"
ko "creds missing" creds_resolve "$_tmp2"
printf 'ORIGA_ADMIN_PASSWORD=secret\n' >"$_tmp2/.env"
eq "creds from .env" "0" "$(
	unset ORIGA_ADMIN_PASSWORD || true
	creds_resolve "$_tmp2"
	echo $?
)"
printf 'ORIGA_ADMIN_PASSWORD=\n' >"$_tmp2/.env"
ko "creds empty value refuses" creds_resolve "$_tmp2"
rm -rf "$_tmp2"

# ---- serve-ui port: parsed from origa_ui/Trunk.toml [serve] (L2: no hand-copied sync point) ----
_tmp3=$(mktemp -d)
printf '[build]\ntarget = "x"\nport = 99\n\n[serve]\nport = 8081\nopen = false\n' >"$_tmp3/Trunk.toml"
eq "serve port from [serve], [build] port ignored" "8081" "$(serve_ui_port_from "$_tmp3/Trunk.toml")"
printf '[serve]\nport = 1420\n' >"$_tmp3/Trunk.toml"
eq "serve port plain" "1420" "$(serve_ui_port_from "$_tmp3/Trunk.toml")"
eq "serve port missing file falls back" "1420" "$(serve_ui_port_from "$_tmp3/nope.toml")"
printf 'garbage content\n' >"$_tmp3/Trunk.toml"
eq "serve port garbage falls back" "1420" "$(serve_ui_port_from "$_tmp3/Trunk.toml")"
rm -rf "$_tmp3"

printf '\n%d passed, %d failed\n' "$_pass" "$_fail"
[ "$_fail" -eq 0 ]
