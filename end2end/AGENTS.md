# AGENTS.md — Origa E2E Tests (`end2end/`)

Playwright E2E tests for Origa. TypeScript + `@playwright/test`.

## npm Scripts

```bash
npm test                     # All tests (headless; pretest runs bddgen)
npm run test:ui              # Playwright UI mode
npm run test:headed          # Visible browser
npm run test:debug           # Debug mode
npm run report               # HTML report (0.0.0.0:9323)
npm run bdd:gen              # Generate Playwright tests from .feature files
npm run test:bdd             # bddgen + run BDD project only
npm run guard:matrix         # Verify the CI e2e matrix covers every scenario
npm run typecheck            # TypeScript type check (tsc --noEmit)
```

## Structure

```text
end2end/
├── .env                                    # TrailBase URL, admin creds
├── config.ts                              # Reads .env, exports getTrailBaseUrl()
├── global-setup.ts                        # Starts TrailBase, sets admin password
├── global-teardown.ts                     # Kills TrailBase process
├── playwright.config.ts                   # Playwright config + webServer definitions
├── bdd/features/                          # Russian Gherkin .feature files
├── bdd/steps/                             # Step definitions (reuse Page Objects)
├── bdd/fixtures.ts                        # BDD test base: testUser/page fixtures
├── pages/ {base,login,...}.page.ts → index.ts
├── fixtures/                              # admin tokens, test data
├── helpers/                               # Navigation, auth, HTTP, cleanup, paths
├── shots/                                 # Screenshot tool: catalog, scenarios, presets
├── screenshots-spec/                      # Playwright entry points of the scenarios
├── scripts/shots.ts                       # Screenshot CLI, scripts/check-ci-matrix.mjs
```

## Local Infrastructure

Playwright auto-manages all infrastructure — no manual setup needed.

| Service | Port | Managed by | Config |
|---------|------|-----------|--------|
| TrailBase | 4000 | `global-setup.ts` | Spawns `trail run --dev` |
| CDN (static files) | 8080 | `playwright.config.ts` webServer | `npx serve ../cdn` |
| App (trunk/wasm) | 1420 | `playwright.config.ts` webServer | `trunk serve` or `npx serve` |

**Just run:** `npx playwright test` — everything starts automatically.

**Required `.env`:**

```ini
ORIGA_ADMIN_EMAIL=admin@localhost
ORIGA_ADMIN_PASSWORD=secret
TRAILBASE_URL=http://127.0.0.1:4000
ORIGA_CDN_BASE_URL=http://localhost:8080
```

**Flow:**

1. `playwright.config.ts` webServer starts CDN (8080) and app (1420)
2. `global-setup.ts` kills stale TrailBase, sets admin password, starts fresh TrailBase (4000)
3. Tests run against all three services
4. `global-teardown.ts` kills TrailBase process

## Screenshot Tool

`end2end/shots/` owns every UI screenshot of the repository. The CLI is
the entry point:

```bash
npm run shots -- list                                    # the catalog of screens
npm run shots -- capture --scenario home --label issue-808 [--upload]
npm run shots -- index                                   # rebuild the registry view
```

`capture` checks the environment first (`shots/preflight.ts`): a missing
`cdn/fonts/`, a poisoned `dist/` or an output directory inside the
repository fail the run before a pixel is written. Then the Playwright
driver (`screenshots-spec/capture.spec.ts`) takes the screenshots, and the
CLI writes the captures into the registry
(`docs/ui-screenshots/shots/`) and prints the markdown block for a pull
request body.

**Do not write a new capture script.** A new screen is a new entry in the
catalog:

- `shots/catalog.ts` — the metadata of the screen (id, title, route,
  locale, viewports, data state). The CLI reads it without loading the
  browser-side modules.
- `shots/scenarios/<id>.ts` — the steps that reach the screen. It
  spreads its catalog descriptor and adds a `run` step.
- `shots/presets/<id>.ts` — the data state the screen is shown in (a
  fresh profile, an imported corpus, a seeded ledger). The set is open by
  design.
- `shots/scenarios/index.ts` — the map of the implementations.

The screenshots-spec files are thin wrappers around that catalog: they
exist so a screen can also be captured by hand.

```bash
npx playwright test --config=playwright.screenshots.config.ts home-insights
```

The runner shares the BDD infrastructure: `global-setup.ts` starts
TrailBase in `end2end/trailbase-fixture` and sets the admin password, the
CDN is served on 8080 (or the production CDN when there is no local
corpus), the app on 1420 (`trunk serve` locally, the built
`origa_ui/dist` when `CI=true`). The files land in
`/tmp/opencode/shots/<label>/` — never in the repository (issue #797).

The suite is run by hand for now: its infrastructure no longer depends
on the developer's machine, but the onboarding flow of a fresh profile is
not verified in a pipeline yet (ADR-066, "Not Done Here"; ADR-067). The
weekly job follows that work.

### Screen examiner (`--examine`, pre-release manual testing)

`capture --examine` sends every captured screen through a local d3
decision model (Ollama, `/v1/systemone`): the model answers typed
yes/no questions about static visual defects (layout overlap, clipping,
tofu glyphs, brand fonts, empty states) with probabilities. Behavior,
hover and flow defects are out of scope — the examiner sees a still
screenshot.

```bash
cd end2end
OLLAMA_URL=http://localhost:11434 npm run shots -- capture --scenario home --label rc-0.7.1 --examine
npm run shots -- capture --scenario home --label rc-0.7.1 --examine --upload --file-issues
npm run exam:canary   # proves the examiner catches defects (needs live Ollama, never CI)
```

- `shots/examiner/model.json` — the test model (which screens get
  examined). Committed: the knowledge is versioned next to the code.
  A screen absent from the model is skipped, not an error.
- `shots/examiner/questions.ts` — the defect checklist (clear distinct
  criteria matter: the model only answers what it is asked).
- Traces: `exam.<scenario>.jsonl` + `exam-report.md` next to the
  captures (outside the repo). A finding is a probability, not a
  verdict — every flag needs manual triage.
- `--file-issues` requires `--upload` (an issue needs public evidence);
  dedup by `fingerprint:` line, label `ai-manual-test`.
- Runtime failures of the model are recorded per screen and fail the
  run loudly; the captures themselves survive.
- The canary (`screenshots-spec/exam-canary.spec.ts`) injects synthetic
  defects into the live home screen and must catch each one in two
  consecutive passes, with the clean screen staying unflagged. The
  measured blind spots of the current d3 builds (clipping, misalignment,
  typeface) are documented in the spec header and in
  `questions.ts` — those questions stay out of the committed screen
  model until a model or formulation clears its canary. It is NOT part
  of `npm test` or the CI matrix — it needs a live Ollama.

## Test Users

| User | Email | Password | Purpose |
|------|-------|----------|---------|
| Admin | `admin@localhost` | `secret` | API operations, user management |
| Manual test | `uwuwu@uwuwu.net` | `uwuwu` | Manual UI testing (pre-created) |
| E2E auto-created | `e2e-<ts>-<rand>@origa.local` | `e2e-test-password-123` | Created/deleted per test |

E2E tests use `setupTestUser()` (in `helpers/auth.ts`) which creates a fresh user via admin API before each test and cleans up after.

## Page Object Pattern

All tests **must** use Page Objects from `pages/`. Select via `[data-testid="..."]` only — never CSS classes or text.

```typescript
import { Page } from '@playwright/test';
import { TestUser } from '../config';
export class LoginPage {
  constructor(private page: Page) {}
  async login(user: TestUser) {
    await this.page.fill('[data-testid="username"]', user.username);
    await this.page.fill('[data-testid="password"]', user.password);
    await this.page.click('[data-testid="login-button"]');
  }
}
```

## BDD Tests (Gherkin)

Complex multi-step user journeys can be described in Russian Gherkin `.feature` files under `bdd/features/`. Step definitions live in `bdd/steps/` and reuse Page Objects.

### When to use Gherkin vs native Playwright

**Everything lives in Gherkin.** Native `.spec.ts` files were decommissioned
(the last three were migrated in the #445 cleanup): multi-step journeys,
single-page DOM checks and technical regressions are all `.feature` scenarios.
Technical mechanisms that don't read well as Gherkin (route gates, request
aborts, bounded click loops) live in `helpers/` and are invoked from steps.

### Pipeline

1. Write `.feature` file in `bdd/features/` (Russian, `# language: ru`).
2. Write step definitions in `bdd/steps/` — import `{ Given, When, Then }` from `bdd/fixtures.ts`.
3. Run `npm run bdd:gen` to generate Playwright tests into `.features-gen/`.
4. Run `npm run test:bdd` (or `npx playwright test --project=bdd`).

**Before `npm test` (all projects)**: run `npm run bdd:gen` first — the BDD project expects generated files in `.features-gen/`.

### Conventions

- `.feature` language: Russian (`# language: ru`). Keywords: `Функция`, `Сценарий`, `Допустим` (Given), `Когда` (When), `Тогда` (Then), `И` (And).
- Domain terms in steps must match [`origa/docs/glossary.md`](../origa/docs/glossary.md).
- Step definitions reuse Page Objects from `pages/`. Do not use raw selectors in steps.
- The BDD fixture (`bdd/fixtures.ts`) provides an authenticated `page` bound to the per-test `testUser` account (fresh e2e user, auto-cleanup).
- **When = action, Then = assertion.** Don't hide assertions in When steps. The `.feature` reader sees what's verified in the Then line.
- **Migration fidelity.** When converting `.spec.ts` → `.feature`, ALL original assertions must be preserved. Self-check against source.
- **Dead step detection.** After removing a step from `.feature`, grep for unconsumed definitions in `bdd/steps/`. `bddgen`/ESLint do NOT catch orphans.
- **Shared helpers.** Reusable test logic lives in `helpers/`. Both `.spec.ts` and `bdd/steps/*.ts` import from there — single source of truth.

## Rules

- Headless by default; use `:headed` / `:ui` scripts for debugging
- `async/await` everywhere — no raw Promises or `.then()`
- Web-first assertions: `await expect(locator).toBeVisible()`
- Tags: `test('name @smoke @critical', ...)`
- Run `npm run typecheck` before committing — ESLint does NOT catch undefined references in TypeScript. Only `tsc --noEmit` catches missing imports.
- `import type` for type-only imports in `shots/**` and `scripts/shots.ts` — the tool runs under Node's type stripping as well as under Playwright, and type stripping does NOT remove a type-only import (it becomes a runtime "does not provide an export named" error that `tsc` stays silent about). ESLint enforces it in that zone.
- Native `*.spec.ts` files are the norm for the screenshot scenarios: they produce artifacts instead of asserting behaviour, so the "everything lives in Gherkin" rule of the BDD suite does not apply to them.
- The scenarios of `shots/` select by `data-testid` — the same rule as the page objects. A raw CSS selector is allowed only where the markup has no marker (a tag inside a card, a checkbox of a drawer), and the reason is a comment next to it.
