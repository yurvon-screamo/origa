#!/bin/sh
# Dev-contour guards for the root Makefile — fail-closed environment checks
# so a local serve/e2e run can never silently bake production endpoints.
# Incident uwuwu#1449: a bare `trunk serve` baked the production TrailBase
# (https://app.origa.uwuwu.net) into the bundle via the ADR-020 fallback and
# wrote records against it; prod answered 403 fail-closed.
#
# The rule behind the guards: the local contour never talks to the production
# TrailBase. The make contour covers web serve, e2e, the e2e dist build and
# `make tauri-dev` (uwuwu#1663). The production CDN is read-only and is the
# documented local content source (root AGENTS.md "Local dev"). Direct
# commands still bake the ADR-020/023 defaults silently — the documented
# remainder of the chosen option A.
#
# Sourced by the root Makefile recipes and by scripts/dev-guards.test.sh.
# POSIX sh on purpose (make recipes run /bin/sh). The pure verdict functions
# (trailbase_is_local, landing_base_present, listener_verdict, corpus_dir)
# are the decision table covered by the test script —
# keep them probe-free so the table stays mechanically testable; all probes
# live in the lower section.
#
# Port defaults below are a deliberate sync point with
# end2end/shots/config.ts (the harness's single source): both honor
# E2E_APP_PORT / E2E_TRAILBASE_PORT / E2E_CDN_PORT. After uwuwu#407 lands the
# harness-layer port guard, re-check there whether this make-layer e2e port
# guard can shrink (both layers stay: different entry points).

# ------------------------------------------------------------------
# Pure decision table (covered by scripts/dev-guards.test.sh)
# ------------------------------------------------------------------

# TRAILBASE_URL verdict: exit 0 = loopback-host URL (local backend);
# exit 1 = empty, non-loopback or malformed. The production default is the
# exact value this guard refuses — an explicit prod host is refused too
# (the norm forbids the local contour talking to prod, not only silent
# fallbacks), mirroring the e2e config which has no prod option at all.
trailbase_is_local() {
	_tb_val=$1
	[ -n "$_tb_val" ] || return 1
	# Absolute http(s) URL only: the client concatenates the base with API
	# paths (trailbase_client.rs), so a schemeless value ("localhost:4000")
	# parses as scheme "localhost:" and every fetch fails — a plausible
	# typo must not pass a strict guard silently. Bare IPv6 authorities are
	# malformed the same way (URL spec requires brackets).
	case $_tb_val in
	https://* | http://*) ;;
	*) return 1 ;;
	esac
	# The decision is made on the URL AUTHORITY only (after the scheme, up
	# to the first /?#): anything after the authority is data, not the host
	# — a query/fragment containing "[::1]" or "localhost" must not flip
	# the verdict.
	_tb_auth=${_tb_val#*://}
	_tb_auth=${_tb_auth%%[/?#]*}
	case $_tb_auth in
	*@*) return 1 ;; # userinfo: credentials are not a host
	esac
	# exact authority forms only: loopback host, optionally a numeric port
	case $_tb_auth in
	localhost | 127.0.0.1 | "[::1]") return 0 ;;
	localhost:* | 127.0.0.1:*)
		_tb_port=${_tb_auth#*:}
		;;
	"[::1]":*)
		# strip through the closing bracket + port colon — the IPv6 colons
		# inside the brackets are part of the host
		_tb_port=${_tb_auth#*]:}
		;;
	*) return 1 ;;
	esac
	case $_tb_port in
	'' | *[!0-9]*) return 1 ;; # empty or non-numeric tail is not a port
	*) return 0 ;;
	esac
}

# Desktop dev value for TRAILBASE_URL (uwuwu#1663). Unset or empty means the
# developer did not choose a backend: the canonical recipe supplies a local
# one instead of letting build.rs bake the ADR-020 production default. An
# explicit loopback value passes through. An explicit non-loopback value
# refuses — the recipe must not silently talk to production.
# Pure: prints the resolved value, exits 1 on a refused value.
tauri_trailbase_value() {
	if [ -z "${1:-}" ]; then
		printf 'http://127.0.0.1:4000\n'
		return 0
	fi
	trailbase_is_local "$1" || return 1
	printf '%s\n' "$1"
}

# Landing build base (root AGENTS.md; origa_landing/build.rs panics without
# it): ORIGA_APP_BASE_URL alone, or the ORIGA_BASE_URI + ORIGA_APP_URI_PREFIX
# pair — the build script accepts either form.
landing_base_present() {
	[ -n "$1" ] && return 0
	[ -n "$2" ] && [ -n "$3" ]
}

# One listener's verdict; exit 0 = the run may proceed, 1 = refuse.
# GUARD_WHY is set to the short reason on refusal.
#   $1 scope:       serve-ui | e2e
#   $2 port role:   app | trailbase | cdn
#   $3 ownership:   own (inside this checkout) | stranger | unknown (no pid/cwd)
#   $4 env verdict: local | notlocal | unverified | -  (app+own only)
listener_verdict() {
	_lv_scope=$1
	_lv_role=$2
	_lv_own=$3
	_lv_env=$4
	GUARD_WHY=""
	if [ "$_lv_scope" = "serve-ui" ]; then
		# trunk serve binds the port itself — any pre-existing listener is
		# the bare "address already in use" the incident hit; refuse with
		# attribution instead of letting trunk fail undiagnostically.
		GUARD_WHY="порт занят, trunk serve не переиспользует слушатель"
		return 1
	fi
	# e2e below. A listener we cannot attribute stays silent — parity with
	# the shots preflight (runbook "Tests in a Fresh Worktree", 09.11).
	if [ "$_lv_own" = "unknown" ]; then
		return 0
	fi
	if [ "$_lv_own" = "stranger" ]; then
		GUARD_WHY="слушатель из чужого чекаута — playwright переиспользует его как свой"
		return 1
	fi
	if [ "$_lv_role" = "app" ]; then
		# Own server on the app port is the documented fast path, but only
		# when its bundle provably points at a local TrailBase (verified
		# via /proc/<pid>/environ). Anything else is the incident class.
		if [ "$_lv_env" = "local" ]; then
			return 0
		fi
		GUARD_WHY="свой сервер собран с непроверенным или прод TrailBase"
		return 1
	fi
	# own trailbase: the harness kills stale fixtures on its own port;
	# own cdn: read-only corpus serving — both legit reuse.
	return 0
}

# ------------------------------------------------------------------
# Probes (impure; wire the pure table to the machine)
# ------------------------------------------------------------------

GUARD_REPO_ROOT=${GUARD_REPO_ROOT:-$(pwd -P)}

# The port trunk serve binds — parsed from origa_ui/Trunk.toml [serve] (the
# single source; trunk does not read the harness's E2E_APP_PORT override).
# Falls back to the known default when the file is unreadable or moved.
serve_ui_port_from() {
	_spf_val=$(sed -n '/^\[serve\]/,/^\[/p' "$1" 2>/dev/null | grep -o 'port[[:space:]]*=[[:space:]]*[0-9][0-9]*' | grep -o '[0-9][0-9]*' | head -n 1)
	if [ -n "$_spf_val" ]; then
		printf '%s\n' "$_spf_val"
	else
		printf '1420\n'
	fi
	return 0
}

serve_ui_port() {
	serve_ui_port_from "$GUARD_REPO_ROOT/origa_ui/Trunk.toml"
}

guard_fail() {
	printf 'GUARD FAIL: %s\n' "$1" >&2
	exit 1
}

guard_warn() {
	printf 'GUARD WARNING: %s\n' "$1" >&2
}

# Listener pids of a port (ss), each as "<pid>|<cwd>", dual-stack deduped by
# pid (the shots preflight prints such pairs twice — #958; this copy must
# not inherit that).
list_listeners() {
	ss -tlnpH "sport = :$1" 2>/dev/null | awk '
        match($0, /pid=[0-9]+/) {
            pid = substr($0, RSTART + 4, RLENGTH - 4)
            if (!(pid in seen)) { seen[pid] = 1; print pid }
        }' | while IFS= read -r _ll_pid; do
		_ll_cwd=$(readlink "/proc/$_ll_pid/cwd" 2>/dev/null)
		printf '%s|%s\n' "$_ll_pid" "$_ll_cwd"
	done
}

is_inside_repo() {
	case $1 in
	"$GUARD_REPO_ROOT" | "$GUARD_REPO_ROOT"/*) return 0 ;;
	*) return 1 ;;
	esac
}

# Why the port pass is OK — attributed facts only (env-check OK lines must
# not claim more than the guard knows; an unattributable listener is the
# documented silence case, reported as such).
listener_ok_note() { # $1 port, $2 role
	_ln_first=$(list_listeners "$1" | head -n 1)
	_ln_pid=${_ln_first%%|*}
	_ln_cwd=${_ln_first#*|}
	if [ -n "$_ln_cwd" ] && is_inside_repo "$_ln_cwd"; then
		case $2 in
		app) printf 'свой сервер (pid=%s), TrailBase верифицирован loopback — легитимный reuse\n' "$_ln_pid" ;;
		trailbase) printf 'свой fixture (pid=%s) — харнес сам убирает stale-инстанс\n' "$_ln_pid" ;;
		cdn) printf 'свой слушатель (pid=%s) — раздача корпуса\n' "$_ln_pid" ;;
		esac
	else
		printf 'слушатель не атрибутируем — гард молчит (паритет с префлайтом)\n'
	fi
}

# The TRAILBASE_URL env verdict of a running process (its bundle is what a
# reuse would serve): local | notlocal | unverified. Note the empty value is
# "notlocal" in effect — ADR-020 treats empty exactly like unset (prod
# fallback).
proc_trailbase_verdict() {
	_pt_file="/proc/$1/environ"
	_pt_val=""
	if [ -r "$_pt_file" ]; then
		_pt_val=$(tr '\0' '\n' <"$_pt_file" | sed -n 's/^TRAILBASE_URL=//p' | head -n 1)
	fi
	if [ -z "$_pt_val" ]; then
		printf 'unverified\n'
	elif trailbase_is_local "$_pt_val"; then
		printf 'local\n'
	else
		printf 'notlocal\n'
	fi
}

# Corpus dir by the serve chain of end2end/shots/config.ts resolveCdnSource:
# ORIGA_CDN_CACHE_DIR ?? ORIGA_CDN_DIR ?? ~/origa-cdn — the `??` semantics
# are set-or-unset (an explicitly SET-EMPTY var does NOT fall through, it
# just fails the existence check), then existence decides. Prints the dir
# and exits 0 when a live corpus exists; exits 1 otherwise (content comes
# from the production CDN — documented, read-only). The exit code IS the
# predicate for the callers (warn_corpus, guard_e2e, env_check).
corpus_dir() {
	_cd_dir=""
	if [ "${ORIGA_CDN_CACHE_DIR+set}" = "set" ]; then
		_cd_dir=$ORIGA_CDN_CACHE_DIR
	elif [ "${ORIGA_CDN_DIR+set}" = "set" ]; then
		_cd_dir=$ORIGA_CDN_DIR
	else
		_cd_dir=${HOME:-}/origa-cdn
	fi
	if [ -n "$_cd_dir" ] && [ -d "$_cd_dir" ]; then
		printf '%s\n' "$_cd_dir"
		return 0
	fi
	return 1
}

# Check every listener of one port against the table; on the first refusal
# print the attributed message and return 1 (caller decides exit/reporting).
check_listeners() {
	_cl_scope=$1
	_cl_role=$2
	_cl_port=$3
	_cl_label=$4
	while IFS='|' read -r _cl_pid _cl_cwd; do
		[ -n "$_cl_pid" ] || continue
		_cl_own=unknown
		if [ -n "$_cl_cwd" ]; then
			if is_inside_repo "$_cl_cwd"; then
				_cl_own=own
			else
				_cl_own=stranger
			fi
		fi
		case $_cl_role in
		app) _cl_env=$(proc_trailbase_verdict "$_cl_pid") ;;
		*) _cl_env="-" ;;
		esac
		if ! listener_verdict "$_cl_scope" "$_cl_role" "$_cl_own" "$_cl_env"; then
			printf 'GUARD FAIL: %s занят (pid=%s, cwd=%s) — %s\n' \
				"$_cl_label" "$_cl_pid" "${_cl_cwd:-?}" "$GUARD_WHY" >&2
			if [ "$_cl_scope" = "e2e" ]; then
				# the port override first — killing a stranger's process (e.g.
				# the machine's llama-server) is the last resort, shots-preflight
				# precedent (end2end/shots/preflight.ts)
				printf '  Remedy: уведите цели — E2E_APP_PORT/E2E_TRAILBASE_PORT/E2E_CDN_PORT=<свободный> — или остановите процесс (kill %s)\n' "$_cl_pid" >&2
			else
				printf '  Remedy: остановите процесс (kill %s)\n' "$_cl_pid" >&2
			fi
			return 1
		fi
	done <<_GUARD_LISTENERS
$(list_listeners "$_cl_port")
_GUARD_LISTENERS
	return 0
}

# Fixture creds resolve: shell env or end2end/.env — the harness contract
# (config.ts dotenv-overrides, preflight checks ORIGA_ADMIN_PASSWORD). The
# password is the load-bearing variable; a non-empty value is required.
creds_resolve() {
	if [ -n "${ORIGA_ADMIN_PASSWORD:-}" ]; then
		return 0
	fi
	grep -q '^ORIGA_ADMIN_PASSWORD=..*' "$1/.env" 2>/dev/null
}

# ------------------------------------------------------------------
# Guards used by the Makefile (each exits non-zero on refusal)
# ------------------------------------------------------------------

# TRAILBASE_URL: strict contract — set AND loopback. An explicit production
# host is refused too: local web dev must not talk to prod TrailBase.
guard_trailbase() {
	if trailbase_is_local "${TRAILBASE_URL:-}"; then
		return 0
	fi
	if [ -n "${TRAILBASE_URL:-}" ]; then
		_gt_why="указывает не на loopback-хост (прод-TrailBase запрещён в локальном контуре)"
	else
		_gt_why="не задана — build.rs молча запечёт прод (ADR-020 fallback, инцидент uwuwu#1449)"
	fi
	guard_fail "TRAILBASE_URL $_gt_why.
  Remedy:
    export TRAILBASE_URL=http://127.0.0.1:4000
    cd end2end && npm run trailbase:start   # поднять локальный TrailBase
  Норма: локальная web-разработка/e2e не смотрит в прод (root AGENTS.md)."
}

guard_cdn_env() {
	[ -n "${ORIGA_CDN_BASE_URL:-}" ] || guard_fail "ORIGA_CDN_BASE_URL не задана.
  Remedy: export ORIGA_CDN_BASE_URL=https://s3.origa.uwuwu.net  (прод-CDN read-only — документированный путь локальной разработки)."
}

guard_landing_base() {
	if landing_base_present "${ORIGA_APP_BASE_URL:-}" "${ORIGA_BASE_URI:-}" "${ORIGA_APP_URI_PREFIX:-}"; then
		return 0
	fi
	guard_fail "ORIGA_APP_BASE_URL не задана (и пары ORIGA_BASE_URI + ORIGA_APP_URI_PREFIX нет) — origa_landing/build.rs паникует.
  Remedy: export ORIGA_APP_BASE_URL=https://app.origa.uwuwu.net"
}

guard_trunk() {
	command -v trunk >/dev/null 2>&1 || guard_fail "trunk не найден в PATH.
  Remedy: cargo install trunk --locked"
}

guard_npm() {
	# SC2016: $HOME/$PATH in the remedy text are a literal copy-paste
	# template for the user, not expressions; single quotes keep shfmt's
	# canonical form (it collapses the double-quoted equivalent back).
	# shellcheck disable=SC2016
	command -v npm >/dev/null 2>&1 || guard_fail 'npm не найден в PATH (в контуре фабрики он в ~/.local/share/pnpm/bin).
  Remedy: export PATH="$HOME/.local/share/pnpm/bin:$PATH"  (ранбук "Tests in a Fresh Worktree", 22.11).'
}

guard_e2e_env() {
	if ! creds_resolve end2end; then
		guard_fail "креды фикстуры TrailBase не резолвятся: нет ORIGA_ADMIN_PASSWORD в shell и в end2end/.env.
  Remedy: cp end2end/.env.example end2end/.env  (и задай пароль) — контракт .env см. end2end/AGENTS.md."
	fi
	[ -d end2end/node_modules ] || guard_fail 'end2end/node_modules отсутствует.
  Remedy: cd end2end && npm ci  (симлинк из основного чекаута ненадёжен — ранбук "Tests in a Fresh Worktree", правка 26.11).'
}

# Corpus presence: skipping corpus tests by design is fine (root AGENTS.md
# "Пропуск вместо паники"), the run just deserves a loud notice.
warn_corpus() {
	if ! corpus_dir >/dev/null; then
		guard_warn "локальный корпус не найден (${ORIGA_CDN_DIR:-не задана} / ~/origa-cdn) — corpus-тесты скипнутся ([skip] маркеры), контент пойдёт с прод-CDN.
  Remedy (сид один раз на машину): aws s3 sync s3://<bucket> ~/origa-cdn --profile origa  (ранбук \"Tests in a Fresh Worktree\")."
	fi
}

# e2e dist build (uwuwu#1663): strict loopback TrailBase, non-empty CDN and
# trunk. The landing base is not needed by origa_ui/build.rs, so guard_dist
# deliberately does not call guard_landing_base.
guard_dist() {
	printf 'Канон: ORIGA_CDN_BASE_URL=http://localhost:8080 TRAILBASE_URL=http://127.0.0.1:4000 make dist\n' >&2
	guard_trailbase
	guard_cdn_env
	guard_trunk
}

# Desktop contour (uwuwu#1663). cargo-tauri is a cargo subcommand binary;
# probing with `cargo` would touch the acceptance delegate marker. The CDN
# guard is absent on purpose: ADR-020/023 allow the desktop fallback to the
# read-only production CDN without env.
guard_tauri_cli() {
	command -v cargo-tauri >/dev/null 2>&1 || guard_fail "cargo-tauri не найден в PATH (cargo tauri dev не запустится).
  Remedy: cargo install tauri-cli --version '^2' --locked"
}

guard_tauri_trailbase() {
	tauri_trailbase_value "${TRAILBASE_URL:-}" >/dev/null || guard_fail "TRAILBASE_URL указывает не на loopback-хост: десктопный рецепт не должен уходить в прод.
  Remedy:
    export TRAILBASE_URL=http://127.0.0.1:4000
    cd end2end && npm run trailbase:start   # поднять локальный TrailBase
  Если переменная не задана, make tauri-dev подставит loopback сам."
}

guard_tauri_dev() {
	guard_tauri_trailbase
	guard_tauri_cli
	guard_trunk
	# beforeDevCommand runs `trunk serve` (tauri.conf.json), which binds the
	# port from Trunk.toml and cannot reuse a listener — same incident class.
	_td_port=$(serve_ui_port)
	check_listeners serve-ui app "$_td_port" "порт serve-ui $_td_port (cargo tauri dev поднимает trunk serve)" || return 1
}

# serve-ui: the incident command. TrailBase strict + CDN non-empty + trunk +
# app port free (ANY listener refuses — trunk serve cannot reuse one).
guard_serve_ui() {
	guard_trailbase
	guard_cdn_env
	guard_trunk
	# trunk serve binds the port parsed from origa_ui/Trunk.toml [serve]; it
	# does NOT read the harness's E2E_APP_PORT override
	_sp_port=$(serve_ui_port)
	check_listeners serve-ui app "$_sp_port" "порт serve-ui $_sp_port" || return 1
}

# e2e: fixture creds + artifacts + tooling + harness ports.
guard_e2e() {
	guard_npm
	guard_e2e_env
	_ep_app=${E2E_APP_PORT:-1420}
	_ep_tb=${E2E_TRAILBASE_PORT:-4000}
	check_listeners e2e app "$_ep_app" "порт приложения $_ep_app" || return 1
	check_listeners e2e trailbase "$_ep_tb" "порт TrailBase $_ep_tb" || return 1
	if corpus_dir >/dev/null; then
		# the harness binds the CDN port only with a live local corpus
		# (playwright webServer is conditional the same way)
		_ep_cdn=${E2E_CDN_PORT:-8080}
		check_listeners e2e cdn "$_ep_cdn" "порт CDN $_ep_cdn" || return 1
	fi
}

guard_just() {
	command -v just >/dev/null 2>&1 || guard_fail "just не найден в PATH.
  Remedy: pacman -S just  (или cargo install just --locked)"
}

# env-check: the full report for a fresh worktree (UC-2). Blocker holes exit
# 1; warnings (corpus) do not.
env_check() {
	_ec_holes=0
	printf '== make env-check: окружение dev-контура (%s) ==\n' "$GUARD_REPO_ROOT"

	if [ -n "${ORIGA_CDN_BASE_URL:-}" ]; then
		printf 'OK   ORIGA_CDN_BASE_URL=%s\n' "$ORIGA_CDN_BASE_URL"
	else
		_ec_holes=$((_ec_holes + 1))
		printf 'HOLE ORIGA_CDN_BASE_URL не задана — export ORIGA_CDN_BASE_URL=https://s3.origa.uwuwu.net (прод-CDN read-only, документированный путь)\n'
	fi

	if landing_base_present "${ORIGA_APP_BASE_URL:-}" "${ORIGA_BASE_URI:-}" "${ORIGA_APP_URI_PREFIX:-}"; then
		printf 'OK   база лендинга: ORIGA_APP_BASE_URL (или пара ORIGA_BASE_URI + ORIGA_APP_URI_PREFIX)\n'
	else
		_ec_holes=$((_ec_holes + 1))
		printf 'HOLE ORIGA_APP_BASE_URL не задана (и пары ORIGA_BASE_URI + ORIGA_APP_URI_PREFIX нет) — origa_landing/build.rs паникует; export ORIGA_APP_BASE_URL=https://app.origa.uwuwu.net\n'
	fi

	if trailbase_is_local "${TRAILBASE_URL:-}"; then
		printf 'OK   TRAILBASE_URL=%s (loopback)\n' "$TRAILBASE_URL"
	elif [ -n "${TRAILBASE_URL:-}" ]; then
		_ec_holes=$((_ec_holes + 1))
		printf 'HOLE TRAILBASE_URL=%s — не loopback (прод-TrailBase запрещён в локальном контуре); export TRAILBASE_URL=http://127.0.0.1:4000\n' "$TRAILBASE_URL"
	else
		_ec_holes=$((_ec_holes + 1))
		printf 'HOLE TRAILBASE_URL не задана (нужна локальному контуру: serve-ui/dist/tauri-dev; без неё бандл печёт прод — ADR-020); export TRAILBASE_URL=http://127.0.0.1:4000\n'
	fi

	if command -v trunk >/dev/null 2>&1; then
		printf 'OK   trunk: %s\n' "$(command -v trunk)"
	else
		_ec_holes=$((_ec_holes + 1))
		printf 'HOLE trunk не найден — cargo install trunk --locked\n'
	fi

	if command -v npm >/dev/null 2>&1; then
		printf 'OK   npm: %s\n' "$(command -v npm)"
	else
		_ec_holes=$((_ec_holes + 1))
		printf "HOLE npm не найден — export PATH=\"\$HOME/.local/share/pnpm/bin:\$PATH\" (ранбук 22.11)\n"
	fi

	if creds_resolve end2end; then
		printf 'OK   креды фикстуры резолвятся (shell или end2end/.env)\n'
	else
		_ec_holes=$((_ec_holes + 1))
		printf 'HOLE креды фикстуры не резолвятся — cp end2end/.env.example end2end/.env (контракт end2end/AGENTS.md)\n'
	fi

	if [ -d end2end/node_modules ]; then
		printf 'OK   end2end/node_modules на месте\n'
	else
		_ec_holes=$((_ec_holes + 1))
		printf 'HOLE end2end/node_modules отсутствует — cd end2end && npm ci (симлинк из основного чекаута ненадёжен: ранбук, правка 26.11)\n'
	fi

	_ec_corpus=$(corpus_dir)
	if [ -n "$_ec_corpus" ]; then
		printf 'OK   корпус: %s\n' "$_ec_corpus"
	else
		guard_warn "корпус не найден (${ORIGA_CDN_DIR:-не задана} / ~/origa-cdn) — corpus-тесты скипнутся, e2e возьмёт контент с прод-CDN; сид: aws s3 sync s3://<bucket> ~/origa-cdn --profile origa"
	fi

	# ports: serve-ui binds its own pinned port (any listener is a hole),
	# the e2e harness ports follow the stranger/own-env verdicts
	_sp_port=$(serve_ui_port)
	if [ -n "$(list_listeners "$_sp_port")" ]; then
		_ec_holes=$((_ec_holes + 1))
		check_listeners serve-ui app "$_sp_port" "порт serve-ui $_sp_port" || true
	else
		printf 'OK   порт serve-ui %s свободен\n' "$_sp_port"
	fi
	if [ "${E2E_APP_PORT:-1420}" != "$_sp_port" ]; then
		if [ -n "$(list_listeners "${E2E_APP_PORT:-1420}")" ]; then
			if check_listeners e2e app "${E2E_APP_PORT:-1420}" "порт e2e-приложения ${E2E_APP_PORT:-1420}"; then
				printf 'OK   порт e2e-приложения %s: %s\n' "${E2E_APP_PORT:-1420}" "$(listener_ok_note "${E2E_APP_PORT:-1420}" app)"
			else
				_ec_holes=$((_ec_holes + 1))
			fi
		else
			printf 'OK   порт e2e-приложения %s свободен\n' "${E2E_APP_PORT:-1420}"
		fi
	fi
	if [ -n "$(list_listeners "${E2E_TRAILBASE_PORT:-4000}")" ]; then
		if check_listeners e2e trailbase "${E2E_TRAILBASE_PORT:-4000}" "порт TrailBase ${E2E_TRAILBASE_PORT:-4000}"; then
			printf 'OK   порт TrailBase %s: %s\n' "${E2E_TRAILBASE_PORT:-4000}" "$(listener_ok_note "${E2E_TRAILBASE_PORT:-4000}" trailbase)"
		else
			_ec_holes=$((_ec_holes + 1))
		fi
	else
		printf 'OK   порт TrailBase %s свободен\n' "${E2E_TRAILBASE_PORT:-4000}"
	fi
	if ! corpus_dir >/dev/null; then
		printf 'OK   порт CDN %s не актуален (корпуса нет — харнес не биндит)\n' "${E2E_CDN_PORT:-8080}"
	elif [ -z "$(list_listeners "${E2E_CDN_PORT:-8080}")" ]; then
		printf 'OK   порт CDN %s свободен\n' "${E2E_CDN_PORT:-8080}"
	elif check_listeners e2e cdn "${E2E_CDN_PORT:-8080}" "порт CDN ${E2E_CDN_PORT:-8080}"; then
		printf 'OK   порт CDN %s: %s\n' "${E2E_CDN_PORT:-8080}" "$(listener_ok_note "${E2E_CDN_PORT:-8080}" cdn)"
	else
		_ec_holes=$((_ec_holes + 1))
	fi

	if [ "$_ec_holes" -gt 0 ]; then
		printf '== %s дыр окружения — закрой по remedy выше (детали: ранбук "Tests in a Fresh Worktree") ==\n' "$_ec_holes"
		exit 1
	fi
	printf '== окружение собрано ==\n'
}
