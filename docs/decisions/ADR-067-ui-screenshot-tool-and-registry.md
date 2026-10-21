# ADR-067: UI screenshot tool and screenshot registry

## Status

Accepted (issue #800 follow-up, ADR-066)

## Date

2026-10-21

## Context

The factory pipeline asks every agent that touches the UI to deliver
screenshots: desktop 1280×800 and mobile 390×844, uploaded to the CDN and
linked in the issue report and the pull request body
(`.opencode/factory/factory-lib.ts`, the kickoff prompt).

Every agent implemented that request from scratch. At the time of this
decision the repository held three one-off scripts
(`end2end/scripts/pr-screenshots.mjs`, `pr-screenshots-jft.mjs`,
`pr-screenshots-lesson-unification.mjs`) and six Playwright specs
(`end2end/screenshots-spec/`), and all nine carried their own copy of the
same bootstrap: an admin token, a test account, the pre-approval of the
~230 MB download consent, the login retry loop and the onboarding walk.
The cost was not the code itself but the failure modes it hid: the consent
gate that blocks every click, the cold WASM load, the zombie worktree that
serves a stale bundle on port 1420.

The second half of the problem was discovery. The curated screenshots in
`.media/` (91 files, desktop, phone and store formats) were all captured
on 2026-09-01, while `origa_ui/src/` changed as late as 2026-10-21.
Nothing tracked that gap: a marketer picking a screenshot for a store
listing had no way to know that the picture shows a UI from six weeks
ago. The map in `.media/README.md` is a hand-written list of files, not a
registry.

Two decisions were already taken and are not repeated here: the images
themselves do not go into git (issue #797 — binary ballast in the
history), and the freshness of a screenshot is not computed (the owner
explicitly dropped that requirement).

## Decision

**1. The screenshots have one tool, and the tool is a catalog of
scenarios.** `end2end/shots/` owns the capture: the session
(`session.ts` — a fresh account, the consent pre-approved, the login, the
data state of the preset), the sink (`capture.ts` — the file, the hash and
the provenance), the catalog (`catalog.ts` — the metadata of every screen)
and the implementations (`scenarios/` — the steps that reach a screen).
`end2end/scripts/shots.ts` is the CLI (`list`, `capture`, `index`).

A new screen is a new entry in the catalog, not a new script with its own
copy of the login flow. The nine capture flows that existed before this
decision are now scenarios of that catalog:

| Before | Now |
| --- | --- |
| `screenshots-spec/word-examples.spec.ts` | scenario `word-examples` (preset `onboarded`) |
| `screenshots-spec/lesson-example.spec.ts` | scenario `lesson-example` (preset `lesson-ready`) |
| `screenshots-spec/counter-scoring.spec.ts` | scenario `counter-scoring` (preset `scoring-n5`) |
| `screenshots-spec/home-due-counter.spec.ts` | scenario `home-due-counter` (preset `onboarded`) |
| `screenshots-spec/home-insights.spec.ts` | scenario `home-insights` (preset `onboarded-insights`) |
| `screenshots-spec/stitch-session.spec.ts` | scenario `stitch-session` (preset `ledger`) |
| `scripts/pr-screenshots.mjs` | scenario `feedback-form` (preset `with-word`) and `token-popup` (preset `onboarded`) — the script had two modes, a lesson with one word and a catalogue with an imported corpus |
| `scripts/pr-screenshots-jft.mjs` | scenario `jft-card` (preset `irodori-nyuumon`) |
| `scripts/pr-screenshots-lesson-unification.mjs` | scenario `lesson-unification` (preset `with-word`) |

The spec files stay as thin wrappers: they are the Playwright entry point
of a scenario, and a screen can still be captured by hand. The three
one-off scripts are gone — they were not tests (they ran in no CI matrix
and matched no `qlty` test pattern), and their logic lives in the
catalog now.

**2. The environment is checked before the capture, not after.**
`shots/preflight.ts` fails the run before a single pixel is written when
the environment cannot produce a meaningful picture: a missing
`cdn/fonts/` (the bundle would be built without `@font-face` and the
screenshots would be sans-serif — a violation of the design rules of the
repository), a poisoned `dist/` (the trunk live-reload bridge breaks the
WASM load of a statically served bundle), a missing output directory
outside the working tree. It warns — and does not fail — when the local
corpus lags behind the deployment, when a listener on port 1420 comes
from another checkout, and when the tokenizer dictionary is absent.

The build mode has ONE predicate (`SERVE_MODE` in `shots/config.ts`), used
by the preflight and by the Playwright config alike: a local run builds
from source, CI serves the prebuilt bundle. Two predicates would let the
check and the served bundle disagree, and the font check would be bypassed
exactly where it matters.

**3. The CDN key is content-addressed.** The upload path is
`pr-screenshots/{label}/{scenario}--{screen}--{viewport}-{sha8}.png`, and
the object carries `immutable`. The key carries the hash of the file, so
the same content keeps the same URL and a changed capture gets a new one.
This is the doctrine of `scripts/_cdn_cache.py` — `immutable` is only
correct for content-addressed objects; the reuse of a stable key with
`immutable` is the edge-cache poisoning of PR #182, and the previous
factory prompt did exactly that with `issue-N-desktop.png`.

**4. The registry is a directory of small JSON files, and the index is a
view.** `docs/ui-screenshots/shots/{scenario}--{screen}--{viewport}--{locale}.json`
holds one capture each: the image facts (size, hash) and the provenance
(scenario, route, locale, commit, build, content source), plus the link of
the last successful upload. One file per capture means two pull requests
that photograph different screens never touch the same file.

The current snapshot and the last published link live side by side: a
local capture that was never published must not erase the link the
marketing team already uses. `docs/ui-screenshots/README.md` is generated
from those files and says so in its header.

**5. The e2e stack has its own base URL.** `E2E_APP_BASE_URL` (default
`http://localhost:1420`) is the address the tests and the tool navigate
to. `ORIGA_APP_BASE_URL` is NOT read in the e2e layer: the repository
prescribes that variable as the production app URL, and a developer who
follows the local-development instructions of `AGENTS.md` would export it
— sending the whole test suite into production and writing test accounts
into the production TrailBase.

## Alternatives Considered

- **Keep the one-off scripts and only document them.** Rejected: the
  recipes were already described in `end2end/AGENTS.md` and the agents
  wrote their own scripts anyway. A document does not remove the
  bootstrap; the shared module does.
- **A separate screenshot service.** Rejected: the whole capture needs
  the app, the CDN corpus and TrailBase — a service would have to own a
  second copy of that stack.
- **Stitch CLI instead of Playwright.** Rejected: the capture of a
  WASM-SPA is flaky there (the script races the hydration, the
  `--storage` flag is silently dropped for http origins) and the fallback
  is a plain Playwright screenshot anyway.
- **Store the images in git next to the registry.** Rejected: issue
  #797. The registry holds the metadata and the links; the images live in
  the CDN.
- **Compute the staleness of a screenshot.** Rejected by the owner: the
  registry records WHEN a capture was made and FROM WHICH commit, and the
  answer to "is it still current" is a new capture, not a computation.
- **One `registry.json` instead of a directory.** Rejected: two pull
  requests with different screens would conflict on every merge. The
  generated index is the only shared file, and its conflict is resolved
  by regenerating it.

## Consequences and Known Trade-offs

- The e2e job and the matrix guard job now run on Node 22: the tool
  executes through Node's type stripping (Node >= 22.18) and the unit
  tests use `node --test` on TypeScript sources.
- The end2end linter covers the new zone (`shots`, `scripts/shots.ts`).
  The `screenshots-spec/` directory is still outside its scope — the
  specs there are thin wrappers now, but the historical debt stays.
- A capture of a scenario with a heavy preset (the N4 or N5 import) runs
  the preset once per viewport. The counter card of the assessment walks
  the whole card queue, so that capture is slow by nature.
- Three of the old specs had rotted: they waited for a navigation link
  named "Words" (the interface is in Russian), they looked for the word
  slide of a hand that starts with the grammar and kanji cards, and the
  example line of a slide loads from the CDN asynchronously. The
  verification run of the migrated scenarios found all three; the
  scenarios wait for the markers of the screen itself now.
- The `end2end/screenshots/` directory is no longer tracked. The five
  PNGs that were in it (2.1 MB) are untracked with a backup, in line
  with issue #797.
- The generated index is a shared file: on a merge conflict it is
  regenerated (`npm run shots -- index`) instead of merged.

## Not Done Here

- **A weekly job that refreshes the registry.** ADR-066 left it open: the
  onboarding flow of a fresh profile is still not verified in a
  CI-like environment. The tool is local and on-demand for now.
- **A staleness verdict.** The registry records the capture date and the
  commit; nothing compares them with the current state of the code.
- **The content source of a capture from a worktree without a corpus.**
  Such a capture reads the production CDN, and the registry says so in
  `cdnBaseUrl` — but the picture then shows the production content, not
  the content of the branch.
- **The unit tests of the factory.** `.opencode/factory` has its own
  test suite, and it runs locally (`bun test`) — not in CI.
- **The eslint coverage of `end2end/screenshots-spec/`.**
