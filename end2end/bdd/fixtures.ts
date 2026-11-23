import { type BrowserContext, type Page } from "@playwright/test";
import { test as base, createBdd } from "playwright-bdd";
import { getAppBaseUrl } from "../config";
import {
  apiLogin,
  setupTestUser,
  uiLogin,
  waitForLoginComplete,
  type TestUserContext,
} from "../helpers/auth";

/**
 * BDD test base with an authenticated page fixture + the matching test account.
 *
 * Extends playwright-bdd's base (NOT @playwright/test) so that createBdd()
 * can bind step definitions. Two fixtures are provided:
 *
 * - `testUser` — creates a fresh test account once per test (setupTestUser)
 *   and exposes its email/password so steps can re-login into the SAME
 *   identity after a logout. Cleanup runs on teardown.
 * - `page` — opens a browser context seeded with the account's session via
 *   `apiLogin` (localStorage seed, no UI form — #1449) and hands over the
 *   page after the authenticated boot settles on the terminal route. The
 *   login FORM stays under test in auth.feature, whose scenarios log in
 *   through `uiLogin` after a logout wipe.
 *
 * Most steps destructure only `{ page }`; re-login flows add `testUser`.
 *
 * Step definitions import { Given, When, Then } from this file.
 */
export const test = base.extend<
  {
    testUser: TestUserContext;
    acqTrainingLog: string[];
    acqKnownCard: { value: string | null };
    acqTargetCard: { value: string | null };
    loginGateRelease: { release: (() => Promise<void>) | null };
    cdnRequestLog: string[];
    apiRequestLog: string[];
    offlineNetLog: { requests: string[]; okResponses: string[] };
    secondDevicePage: Page;
    oauthTauriSession: {
      page: Page | null;
      context: BrowserContext | null;
      tokenRequests: string[];
    };
    oauthConsoleLogs: string[];
  },
  object
>({
  testUser: [
    // Empty destructuring `{}` = "this fixture depends on no other
    // fixtures". Playwright requires the (context, use) signature; an
    // empty object is the idiomatic way to say "no dependencies".
    async ({}, use) => {
      const ctx = await setupTestUser();
      await use(ctx);
      await ctx.cleanup();
    },
    { scope: "test" },
  ],
  page: [
    async ({ browser, testUser }, use) => {
      // Base URL belongs to the context (master's change); the inbox
      // seam init script keeps our origin guard.
      const context = await browser.newContext({ baseURL: getAppBaseUrl() });
      // Opt this browser context into the inbox test seam (slice IN-1):
      // the app registers the window hook only when this exact
      // localStorage flag is present. Origin-guarded: init scripts also
      // run on about:blank where localStorage access is a SecurityError.
      await context.addInitScript((origin) => {
        if (window.location.origin === origin) {
          window.localStorage.setItem("__origa_e2e_seam", "1");
        }
      }, getAppBaseUrl());
      // Authenticated boot without the UI form (#1449): the session is
      // seeded via the API before the first page exists, so the app
      // rehydrates it on boot. The boot's settle wait keeps the same
      // content marker the UI-login path used (waitForLoginComplete).
      await apiLogin(context, testUser.email, testUser.password);
      const page = await context.newPage();
      await page.setViewportSize({ width: 1280, height: 720 });
      try {
        await page.goto(getAppBaseUrl(), { waitUntil: "domcontentloaded" });
        await waitForLoginComplete(page);
        await use(page);
      } finally {
        await context.close();
      }
    },
    { scope: "test", timeout: 180_000 },
  ],
  // Training rotation log: each answer appends the front's data-card-id so
  // Then-steps can assert rotation invariants (per-round card sets,
  // direction switch timing) across separate When-steps.
  acqTrainingLog: [
    async ({}, use) => {
      await use([] as string[]);
    },
    { scope: "test" },
  ],
  // Card remembered before a mark-as-known action (replacement asserts).
  acqKnownCard: [
    async ({}, use) => {
      await use({ value: null as string | null });
    },
    { scope: "test" },
  ],
  // Target card of a selective-answers training step (reopen asserts).
  acqTargetCard: [
    async ({}, use) => {
      await use({ value: null as string | null });
    },
    { scope: "test" },
  ],
  // Release callback of the held login request (spinner test): the
  // "held" When-step stores it here, the "released" step calls it —
  // manual gate instead of a fixed sleep, so the in-flight assert
  // cannot flake on timing.
  loginGateRelease: [
    async ({}, use) => {
      await use({ release: null });
    },
    { scope: "test" },
  ],
  // Request URLs observed during a clean-cache app start (the
  // rkyv fast-path scenario): the When-step that drives the fresh
  // browser context appends every network request here, the Then-step
  // asserts the fallback sources were never fetched.
  cdnRequestLog: [
    async ({}, use) => {
      await use([] as string[]);
    },
    { scope: "test" },
  ],
  // URLs of PATCH requests against the user record, observed by the
  // stress scenarios to assert the sync short-circuit (ADR-045): the
  // When-step attaches a request listener AFTER the measurement window
  // is defined, the Then-step counts entries.
  apiRequestLog: [
    async ({}, use) => {
      await use([] as string[]);
    },
    { scope: "test" },
  ],
  // Offline-startup network log: the When-step that reloads the app
  // under a dead network records CDN-origin requests and successful
  // responses; the Then-steps assert "no successful CDN responses" and
  // the single allowed manifest probe. Aborted requests never fire a
  // `response` event, so only live responses land in `okResponses`.
  offlineNetLog: [
    async ({}, use) => {
      await use({ requests: [] as string[], okResponses: [] as string[] });
    },
    { scope: "test" },
  ],
  // Boot slot for the oauth deep-link routing scenarios (#1482): the Given
  // steps fill it with a page booted inside the `__TAURI__` transport mock
  // (helpers/tauri-mock.ts); the After hook in the steps file closes the
  // context. tokenRequests accumulates every `/api/auth/v1/token` request
  // so the positive-control step can observe the exchange at the network
  // boundary (release builds hide debug logs, but not fetches).
  oauthTauriSession: [
    async ({}, use) => {
      await use({
        page: null as Page | null,
        context: null as BrowserContext | null,
        tokenRequests: [] as string[],
      });
    },
    { scope: "test" },
  ],
  // Console output (ALL levels and types) captured by the oauth mock boot;
  // the routing Then-step pins the error-branch messages by substring.
  // Type-agnostic capture is the point: tracing-wasm emits every tracing
  // level — error! included — through console.log (the crate has no
  // console.error binding), so a type-filtered capture would never see it.
  oauthConsoleLogs: [
    async ({}, use) => {
      await use([] as string[]);
    },
    { scope: "test" },
  ],
  // A second, independent browser context logged into the SAME test
  // account — an honest "new device": its IndexedDB partition is empty,
  // so the first login there exercises the remote→local restore path
  // (ADR-045) rather than the logout dance (whose remote-survival is an
  // accident of operation ordering, not a contract). The context stays
  // open for the whole test; later steps navigate its page.
  secondDevicePage: [
    async ({ browser, testUser }, use) => {
      // Own timeout slot (does NOT touch the test deadline): the
      // login on this device covers the multi-minute restore of a
      // large corpus (remote → local, ADR-045) — the project
      // default 60s aborts it mid-restore.
      const context = await browser.newContext({
        baseURL: getAppBaseUrl(),
        locale: "ru-RU",
      });
      const page = await context.newPage();
      await page.setViewportSize({ width: 1280, height: 720 });
      await uiLogin(page, testUser.email, testUser.password, {
        baseURL: getAppBaseUrl(),
      });
      await use(page);
      await context.close();
    },
    { scope: "test", timeout: 420_000 },
  ],
});

export const { Given, When, Then, Before, After } = createBdd(test);
