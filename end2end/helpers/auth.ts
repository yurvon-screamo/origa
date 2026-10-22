import { type Browser, type Page } from "@playwright/test";
import { getAppBaseUrl } from "../config";
import {
  getAdminToken,
  createTestUser,
  deleteTestUserWithRetry,
} from "../fixtures/admin";

export const DEFAULT_TEST_PASSWORD = "e2e-test-password-123";

/**
 * Unique mailbox part shared by all test-account generators. Keeping one
 * source matters: the `e2e-` prefix is a load-bearing contract with the
 * orphan sweeper (`DELETE FROM user WHERE email LIKE 'e2e-%'`), so every
 * generated address must start with it.
 */
function uniqueMailbox(): string {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substring(2, 8);
  return `e2e-${timestamp}-${random}`;
}

export function generateUniqueEmail(): string {
  return `${uniqueMailbox()}@origa.local`;
}

/**
 * Apple "Hide My Email" relay address for a test account. The relay domain
 * makes the app seed an EMPTY display name (see `utils/display_name.rs`),
 * reproducing the Apple-account onboarding flow.
 */
export function generateRelayEmail(): string {
  return `${uniqueMailbox()}@privaterelay.appleid.com`;
}

/**
 * Wipes ALL client-side auth state (cookies, localStorage, sessionStorage,
 * IndexedDB, cache storage) so the app falls back to the public /login
 * route on the next load. Leptos rehydrates the session from these stores,
 * so clearing cookies alone is not enough.
 */
export async function wipeClientAuthState(page: Page): Promise<void> {
  await page.context().clearCookies();
  await page.goto("/");
  await page.evaluate(async () => {
    try {
      window.localStorage.clear();
      window.sessionStorage.clear();
      if (window.indexedDB && indexedDB.databases) {
        const dbs = await indexedDB.databases();
        for (const db of dbs) {
          if (db.name) indexedDB.deleteDatabase(db.name);
        }
      }
      if (window.caches) {
        const keys = await caches.keys();
        for (const k of keys) await caches.delete(k);
      }
    } catch {
      // ignore — some browsers block storage access in cross-origin iframes
    }
  });
  await page.reload();
}

/**
 * Wait until the app finished the login round-trip.
 *
 * A URL match alone is not a reliable "logged in" signal: the app settles
 * on the ROOT route ("/" renders Home) and never moves to /home, so the
 * terminal-URL wait has to fall back to the content markers — the same
 * fallback the BDD suite uses (see uiLogin below).
 *
 * `timeout` is the budget for the terminal-URL wait. The fallback keeps
 * its own (short) budget: it only runs when the URL did not settle.
 * Callers that boot a cold WASM bundle (local `trunk serve`) need a
 * larger budget than the CI-tuned default.
 */
export async function waitForLoginComplete(
  page: Page,
  timeout = 90_000,
): Promise<void> {
  try {
    await page.waitForURL(/\/(home|onboarding)$/, { timeout });
  } catch {
    const toggle = page.getByTestId("login-password-toggle");
    await toggle.waitFor({ state: "hidden", timeout: 15_000 });
    await page
      .getByTestId("home-page")
      .or(page.getByTestId("onboarding-stepper"))
      .waitFor({ state: "visible", timeout: 60_000 });
  }
}

/**
 * Restore the one-time resource-download consent.
 *
 * The app shows the ~230 MB download overlay on authenticated routes
 * until the consent is given, and the overlay blocks every click. The
 * specs pre-approve it through an init script, but a `localStorage.clear()`
 * wipes the flag again — so it has to be restored after the clear.
 */
export async function restoreDownloadConsent(page: Page): Promise<void> {
  await page.evaluate(() =>
    localStorage.setItem("origa_resource_download_consented", "true"),
  );
}

export interface TestUserContext {
  email: string;
  password: string;
  userUuid: string;
  adminToken: string;
  adminCsrfToken: string;
  cleanup: () => Promise<void>;
}

export async function setupTestUser(options?: {
  email?: string;
  password?: string;
}): Promise<TestUserContext> {
  const email = options?.email ?? generateUniqueEmail();
  const password = options?.password ?? DEFAULT_TEST_PASSWORD;

  const adminAuth = await getAdminToken();
  const userUuid = await createTestUser(
    adminAuth.token,
    adminAuth.csrfToken,
    email,
    password,
  );

  return {
    email,
    password,
    userUuid,
    adminToken: adminAuth.token,
    adminCsrfToken: adminAuth.csrfToken,
    cleanup: async () => {
      await deleteTestUserWithRetry(
        adminAuth.token,
        adminAuth.csrfToken,
        userUuid,
        email,
      );
    },
  };
}

export async function uiLogin(
  page: Page,
  email: string,
  password: string,
  options: { baseURL?: string } = {},
): Promise<void> {
  const maxRetries = 3;
  const baseURL = options.baseURL ?? getAppBaseUrl();
  const appOrigin = new URL(baseURL).origin;

  // The first-run resource-download consent screen (App Review 4.2.3(ii))
  // blocks authenticated routes until the user clicks Download. Tests are
  // not about that screen, so pre-approve the download before any app
  // script runs on the app origin. Matches gloo_storage JSON encoding
  // (`LocalStorage::set(key, true)` stores the literal `true`).
  //
  // The origin is a parameter, not the literal 1420: a scenario may run
  // against another build (a before/after pair on :1421). The write is
  // guarded because a cross-origin document (an OAuth provider) can throw
  // a SecurityError on localStorage access.
  await page.context().addInitScript((origin: string) => {
    if (window.location.origin !== origin) return;
    try {
      window.localStorage.setItem("origa_resource_download_consented", "true");
    } catch {
      // storage blocked on this origin — the consent screen stays, the
      // caller decides whether that is fatal
    }
  }, appOrigin);

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    // The whole login flow is retried, not just waitForURL: the password
    // toggle and the inputs race the WASM cold load too, and a toggle
    // miss used to escape the retry loop as an immediate failure.
    try {
      await page.goto(baseURL, {
        waitUntil: "domcontentloaded",
      });

      // The email/password form is collapsed behind a "Sign in with
      // password" toggle by default (mobile viewport fit). Wait for the
      // toggle to mount (races with WASM load) then expand it before
      // waiting for the inputs.
      const passwordToggle = page.getByTestId("login-password-toggle");
      await passwordToggle.waitFor({ state: "visible", timeout: 60_000 });
      // Explicit action timeouts: Playwright's default actionTimeout is
      // 0 (unbounded) — a stability-blocked click would hang the whole
      // test budget instead of feeding the retry loop below.
      await passwordToggle.click({ timeout: 30_000 });

      await page
        .locator('input[type="email"], input[data-testid="email-input"]')
        .waitFor({ state: "visible", timeout: 30_000 });

      await page.fill(
        'input[type="email"], input[data-testid="email-input"]',
        email,
      );
      await page.fill(
        'input[type="password"], input[data-testid="password-input"]',
        password,
      );
      await page.click(
        'button[type="submit"], button[data-testid="login-submit"]',
        { timeout: 30_000 },
      );

      // The login is complete only when the app navigates to its
      // terminal route: /home (existing onboarded user) or /onboarding
      // (fresh user — the wizard). A "form detached" shortcut proved
      // WRONG: the form unmounts transiently while auth is still in
      // flight, and callers then clicked step buttons on a page that
      // was still the login screen (the 42-minute onboarding-lesson
      // run). One legitimate exception needs handling: the full-corpus
      // restore (#492) settles on the ROOT route ("/" renders Home)
      // without ever matching the terminal URL — for that case accept
      // the root only with explicit CONTENT markers (no login form +
      // home/wizard mounted).
      await waitForLoginComplete(page);
      return;
    } catch (e) {
      if (attempt === maxRetries) {
        throw new Error(
          `Login failed after ${maxRetries} attempts for user ${email}: ${e}`,
          { cause: e },
        );
      }
      // brief cooldown before the retry — WASM may still be loading
      await page.waitForTimeout(3_000);
    }
  }
}

export async function withAuthenticatedPage(
  browser: Browser,
  use: (page: Page) => Promise<void>,
): Promise<void> {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.setViewportSize({ width: 1280, height: 720 });

  const userCtx = await setupTestUser();

  try {
    await uiLogin(page, userCtx.email, userCtx.password);
    await use(page);
  } finally {
    await context.close();
    await userCtx.cleanup();
  }
}
