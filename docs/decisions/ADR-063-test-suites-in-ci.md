# ADR-063: Test suites in CI — landing SSR tests, screenshot specs, coverage

## Status

Accepted (issue #800)

## Date

2026-10-21

## Context

An audit of the test model against the actual CI (cycle 2, 2026-10-19) found
three suites that exist but never run in a pipeline:

1. **`origa_landing` SSR tests — green but invisible.** The Test job runs
   `cargo test --workspace --exclude origa_landing`, and the lint job only
   *checks* the crate (clippy does not link). So the landing package was
   compiled by the landing Docker image alone — without its tests and
   without lints. Three red landing tests had lived on master for weeks
   without CI noticing (sitemap 90≠92, the vi-kanji guard,
   `invalid_lang_param`).
   Count correction: the audit spoke of "137 green tests"; the current
   suite is **352** (69 lib unittests + 283 integration tests: blog 126,
   redirect 35, locale_negotiate 32, seo_meta 31, cache_headers 17,
   docs 16, not_found 12, sitemap 7, security_headers 4, build_config 3).
2. **Screenshot specs — manual only.** Six specs in
   `end2end/screenshots-spec/` were run by hand through
   `playwright.screenshots.config.ts`; no workflow mentioned them.
3. **Coverage — no practice.** `cargo-llvm-cov` was installed on developer
   machines, but no run collected a report, so "what is the coverage of
   this PR" had no answer. `cargo-mutants` was not used at all.

While wiring the landing suite into CI, two concrete blockers surfaced that
the audit could not see from the outside:

**The landing test binaries do not link on current stable.** The project
profile is already `[profile.dev] debug = "line-tables-only"` (with
`[profile.dev.package."*"] debug = false` for external dependencies), yet
the crate still fails to link: `rust-lld` — the default linker on stable
≥ 1.98 — reports `relocation R_X86_64_32 out of range` (full debug info)
or `(.debug_str): section too large` (line tables). The cause is content,
not code: `origa_landing/src/blog/registry.rs` and
`origa_landing/src/docs/registry.rs` embed ~120 markdown files through
`include_str!`, and the resulting debug-info string table is enormous
(rustc peak RSS ≈ 20 GB, crate compile ≈ 10 minutes). The dependencies
are innocent (`libleptos-*.rlib` ≈ 3 MB). Building the same crate with
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` produces a ~100 MB
artifact that links in seconds.

**The screenshot specs could not work outside a developer's machine.**
`playwright.screenshots.config.ts` started TrailBase itself, from the
`end2end/` directory, with `DEPOT=./trailbase-fixture/traildepot` — an
environment variable TrailBase does not read (`trail run --help` lists
`ADDRESS`, `ADMIN_ADDRESS`, … but no depot option; the depot comes from
the working directory). The instance therefore ran on a fresh default
depot: no `user` / `domain_user` record APIs (the app reads its profile
from `/api/records/v1/domain_user`), and an `admin@localhost` account
with a *random* password, while every spec logs in as `admin@localhost`
with a known one. The BDD suite never hit this because it starts
TrailBase through `end2end/global-setup.ts`, which runs in
`end2end/trailbase-fixture` and sets the admin password.

**Two screenshot specs had rotted behind the download-consent gate.** The
first-run resource-download screen (App Review 4.2.3(ii)) blocks every
authenticated route until the user confirms it. The specs pre-approve it
through an init script, but `word-examples.spec.ts` and
`lesson-example.spec.ts` also call `localStorage.clear()` right after the
page opens — which wipes that consent again, so the ~230 MB overlay
reappears and the login never reaches `/home`. `counter-scoring.spec.ts`
had already been hardened against exactly this (issue #808); the other two
were left behind. Both now restore the flag after the clear, like their
hardened sibling.

## Decision

**1. The landing SSR tests run in CI, in their own job.**
`.github/workflows/ci.yml` gains a `landing-test` job
(`needs: [changes]`, gated by the new `run_landing_test` flag: the
`landing` path filter, the `ci` path filter, or any non-PR event). The
`landing` filter also covers `Cargo.toml` and `Cargo.lock`: the landing
crate takes its dependencies from the workspace manifest, so a leptos or
axum bump changes the landing code without touching `origa_landing/**`.
The job runs `cargo test -p origa_landing --features ssr` with
`CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0` — the
workaround for the linker limit — and joins the CI Gate, so a red
landing test blocks the pull request (fail-closed, skip only when
`run_landing_test == false`).

The same job lints the ssr code (`cargo clippy -p origa_landing
--features ssr --all-targets -- -D warnings`): the workspace clippy run
builds the crate WITHOUT the ssr feature, so `src/server.rs`,
`src/main.rs` and the ten ssr integration test targets were never
compiled by any workflow.

A separate job rather than a step in the Test job: the landing compile is
the heaviest in the workspace, so it gets a runner of its own and runs in
parallel with the rest of the pipeline instead of queueing behind the
workspace tests on a shared 16 GB runner.

**2. The screenshot specs stay manual for now — the weekly job comes
after the specs are green.** The infrastructure rot is fixed in this
pull request (see below), but the specs still do not complete their
onboarding flow in a CI-like environment: a fresh profile has to walk
the wizard with the ~230 MB resource download, and the run stalls there
(reproduced locally against a built bundle and a fresh TrailBase
instance). Shipping a weekly job that is red every Monday would be
noise, not a net — so the job is a follow-up, and the decision is
recorded here.

What was fixed, so that the suite runs at all outside a developer's
machine:

- `end2end/playwright.screenshots.config.ts` starts TrailBase through
  the shared `global-setup.ts` / `global-teardown.ts` (the BDD suite's
  mechanism) instead of its own webServer entry, and serves the
  prebuilt `origa_ui/dist` when `CI=true` — the same recipe as
  `end2end/playwright.config.ts`.
- `word-examples.spec.ts` and `lesson-example.spec.ts` restore the
  download consent after their `localStorage.clear()` call, the way
  `counter-scoring.spec.ts` already does — without it the consent
  overlay blocks the login and the spec fails.
- The specs wait for the login with the shared `waitForLoginComplete`
  helper and detect the wizard with `isOnboardingShown`. The URL is not
  a reliable signal in either place: right after the login the app sits
  on the root route, and the redirect of a fresh profile to /onboarding
  happens only once the app is up — so the marker is what decides
  ("home-page" / "onboarding-stepper"), the same way the BDD suite
  decides it.
- `stitch-session.spec.ts` creates its demo account through the admin
  API instead of relying on a pre-existing local user.

**3. Coverage is a weekly trend plus a per-PR diff report, no threshold.**
The same workflow (`weekly-quality.yml`, display name "Quality Coverage")
has a `coverage` job:
`cargo llvm-cov -p origa --lcov --output-path lcov.info --summary-only`,
plus an upload of `lcov.info` (30 days). Publishing to Qlty Cloud is
wired behind the repository secret `QLTY_COVERAGE_TOKEN` (set
2026-10-21, so the weekly trend lands in Qlty Cloud; the Qlty side is
configured in `.qlty/qlty.toml`, `[coverage]`).

The workflow also runs on `pull_request` against master when core paths
change (`origa/**`, `Cargo.toml`, `Cargo.lock`, the workflow file
itself): the same job uploads coverage for the PR HEAD, and Qlty Cloud
renders the diff-coverage PR comment (Project Settings → Comments →
Summary Comments must be enabled once in the Qlty UI). It stays
deliberately outside the CI Gate — information, not a merge blocker. On
fork/dependabot PRs the publish step degrades silently (secrets are not
available there); the artifact upload still works. Concurrency is
grouped per PR, so a fresh push cancels the stale analysis.

**4. `cargo-mutants` is out of scope here.** A calibration run is a
separate task with its own runtime profile.

## Alternatives Considered

- **Add the landing tests to the Test job instead of a separate job.**
  Rejected: the landing compile would serialize after the workspace tests
  and share the runner's memory with them.
- **Fix the linker problem in the project profile**
  (`[profile.dev.package.origa_landing] debug = false` and the same for
  `profile.test`). This is the better fix — it also repairs local builds
  — but it changes `Cargo.toml`, which the repository rules put behind an
  explicit request. Deferred, see below.
- **Run the screenshot specs on every pull request.** Rejected: they need
  the full app stack (WASM build, TrailBase, CDN data) and their purpose
  is producing visual artifacts, not gating a change.
- **Keep the screenshot specs manual and document that.** Accepted as the
  current state, with a caveat: a manual-only suite rots silently — the
  same failure mode this ADR removes for the landing tests. The rot is
  what this pull request fixes (the infra, the consent, the login and
  onboarding detection); what is left is the onboarding flow itself in a
  CI-like environment, and the weekly job follows it. Shipping a job
  that is red every Monday would be noise, not a net.
- **Run coverage on every pull request and publish to Qlty.** Rejected for
  now: llvm-cov rebuilds the whole `origa` dependency graph with
  instrumentation on every run, and with no threshold the per-PR number
  changes no decision. Per-PR coverage is a follow-up, once the Qlty
  token exists.
- **Set a coverage threshold.** Rejected: a threshold without a baseline
  only produces churn.

## Consequences and Known Trade-offs

- A pull request that touches `origa_landing/**` now pays a cold compile
  of the landing dependency graph. The shared rust cache (`origa-ci-v2`)
  is written with `save-if: false` in pull-request jobs, so it only helps
  after a release run has populated it.
- The CI landing job builds without debug info, so backtraces lose line
  numbers. The trade is deliberate: the alternative is a 20 GB compile
  that the runner does not have.
- The screenshot specs are run by hand for now. Their infrastructure no
  longer depends on the developer's machine, but the onboarding flow of
  a fresh profile (wizard + the ~230 MB resource download) is not
  verified in a pipeline yet — the weekly job follows that work.
- The CDN data download is cached by a content-addressed key
  (`cdn-data-v15-<hash of end2end/cdn-manifest.txt>`), and GitHub caches
  are branch-scoped: the weekly run on master reuses only the caches
  created on master. The first weekly run pulls the full manifest
  (~1200 files), later runs hit the cache.
- `cargo llvm-cov` writes no report when a test target fails, so a broken
  `origa` suite means no coverage number that week. The coverage job is
  fail-closed on purpose: the Test job is the gate for test failures.

## Not Done Here (future work)

- Add the weekly screenshot job once the specs complete their onboarding
  flow in a CI-like environment (fresh TrailBase instance, prebuilt
  bundle, CDN data served locally).
- Move the linker workaround into the project profile:
  `[profile.dev.package.origa_landing] debug = false` and the
  `profile.test` twin. Needs a `Cargo.toml` change, so it was left out
  of this pull request.
- Add the `QLTY_COVERAGE_TOKEN` repository secret to switch the Qlty
  publication on, then decide whether coverage also runs per pull
  request.
- Run `cargo-mutants` as a calibration pass over the domain and memory
  crates and record which tests actually catch regressions.
- The `stitch-session` demo account (`insights2@origa.local`) does not
  match the `e2e-` prefix the orphan collector deletes, so it stays in a
  local TrailBase depot forever. Renaming it touches the seeded data of
  the spec and was left out of this pull request.
