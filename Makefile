# Root dev-contour targets (GNU make; Linux contour — factory worktrees and
# the owner's machine). This file only WRAPS canonical commands from
# AGENTS.md / runbooks / package.json and guards the dangerous entries —
# it is not a second source of truth for the commands themselves.
#
# Why guards: incident uwuwu#1449 — a bare `trunk serve` silently baked the
# production TrailBase into the bundle (ADR-020 fallback) and hit prod with
# writes (403 fail-closed). Every serve/e2e entry here runs fail-closed
# checks first (scripts/dev-guards.sh — the same source the decision-table
# tests in scripts/dev-guards.test.sh use). The production CDN is read-only
# and stays the documented local content source; `cargo tauri dev` (desktop)
# is out of the norm's scope (uwuwu#1663).
#
# Targets are additive: CI/release pipelines never invoke make.

SHELL := /bin/sh

# Port overrides must reach the harness delegates (playwright reads them):
# `make e2e E2E_CDN_PORT=8081 ...` flows through to npm. Export only the
# ones actually set — exporting undefined make vars would put EMPTY env
# vars into the delegates, and the harness port parser refuses "" loudly.
ifneq (,$(strip $(E2E_APP_PORT)))
export E2E_APP_PORT
endif
ifneq (,$(strip $(E2E_TRAILBASE_PORT)))
export E2E_TRAILBASE_PORT
endif
ifneq (,$(strip $(E2E_CDN_PORT)))
export E2E_CDN_PORT
endif

GUARDS := scripts/dev-guards.sh

.PHONY: env-check check test serve-ui serve-landing e2e shots css-landing
.PHONY: guard-build-env guard-serve-ui guard-e2e

# --- environment report (fresh-worktree doctor; UC-2) ---
env-check:
	@. $(GUARDS) && env_check

# --- rust dev cycle ---
guard-build-env:
	@. $(GUARDS) && guard_cdn_env && guard_landing_base

check: guard-build-env
	cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings

test: guard-build-env
	@. $(GUARDS) && warn_corpus
	cargo test --workspace

# --- local serve (the incident command, guarded) ---
guard-serve-ui:
	@. $(GUARDS) && guard_serve_ui

serve-ui: guard-serve-ui
	cd origa_ui && trunk serve

serve-landing: guard-build-env
	cargo run --bin origa_landing

# --- e2e / shots / landing css (delegates; no guard duplication) ---
guard-e2e:
	@. $(GUARDS) && guard_e2e

e2e: guard-e2e
	cd end2end && npm test -- $(ARGS)

shots:
	cd end2end && npm run shots -- $(ARGS)

css-landing:
	just --directory origa_landing css
