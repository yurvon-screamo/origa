import { type BrowserContext, type Page } from "@playwright/test";
import { getAppBaseUrl } from "../config";
import {
  hasStoredSession,
  waitForLoginComplete,
  type LoginCompletionOptions,
  type LoginCompletionResult,
} from "./login-completion.ts";
import {
  getAdminToken,
  createTestUser,
  deleteTestUserWithRetry,
  loginTestUser,
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

export {
  hasStoredSession,
  waitForLoginComplete,
  type LoginCompletionOptions,
  type LoginCompletionResult,
  type LoginTerminal,
} from "./login-completion.ts";

/**
 * Restore the one-time resource-download consent.
 *
 * The app shows the ~145 MB download overlay on authenticated routes
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

/**
 * The one-time resource-download consent init script, shared by both login
 * paths. The first-run consent screen (App Review 4.2.3(ii)) blocks
 * authenticated routes until the user clicks Download, and tests are not
 * about that screen — so the consent is pre-approved before any app script
 * runs on the app origin. The origin is a parameter, not the literal 1420:
 * a scenario may run against another build (a before/after pair on :1421).
 * The write is guarded because a cross-origin document (an OAuth provider)
 * can throw a SecurityError on localStorage access.
 */
function addDownloadConsentInitScript(
  context: BrowserContext,
  appOrigin: string,
): void {
  void context.addInitScript((origin: string) => {
    if (window.location.origin !== origin) return;
    try {
      window.localStorage.setItem("origa_resource_download_consented", "true");
    } catch {
      // storage blocked on this origin — the consent screen stays, the
      // caller decides whether that is fatal
    }
  }, appOrigin);
}

/**
 * JWT payload claims the session seed needs. Mirror of `JwtClaims`
 * (origa_ui/src/repository/trailbase_auth.rs): `sub` is the TrailBase user
 * id, `email` is optional, `exp` is Unix seconds.
 */
interface SeededJwtClaims {
  sub: string;
  email?: string;
  exp?: number;
}

/**
 * Decodes the JWT payload segment the way `decode_jwt_claims`
 * (trailbase_auth.rs) does — base64url, no padding. Runs in the Node
 * process of the runner only; inside the browser the app decodes its own.
 */
function decodeJwtPayload(token: string): SeededJwtClaims {
  const parts = token.split(".");
  if (parts.length !== 3) {
    throw new Error(`apiLogin: invalid JWT format (${parts.length} parts)`);
  }
  const base64 = parts[1].replaceAll("-", "+").replaceAll("_", "/");
  const json = Buffer.from(base64, "base64").toString("utf-8");
  const parsed = JSON.parse(json) as Partial<SeededJwtClaims>;
  if (!parsed.sub) {
    throw new Error("apiLogin: JWT payload has no `sub` (trailbase_id)");
  }
  return { sub: parsed.sub, email: parsed.email, exp: parsed.exp };
}

/**
 * Seeded-session login: what `login_with_email_password`
 * (origa_ui/src/repository/trailbase_client.rs) stores after a UI login —
 * without the UI form itself.
 *
 * The app rehydrates the session from `localStorage["trailbase_session"]`
 * (gloo_storage JSON, `origa_ui/src/repository/session.rs`) on boot, so a
 * context whose storage holds the session starts authenticated: the boot
 * lands straight on the terminal route (fresh user → the wizard, restored
 * → home), and the form dance (toggle/fill/submit/auth round-trip/retry
 * loop) disappears from every scenario that only needs an authenticated
 * app. The seeded object mirrors `TrailBaseSession` field-for-field,
 * including `expires_at = claims.exp ?? now + 3600` (the app's fallback,
 * Unix seconds). The locale of the boot-time profile creation is pinned at
 * the config level (`use.locale` in playwright.config.ts) — the same
 * carrier the UI-login path relies on.
 *
 * The seed is a REAL localStorage write, not an init script: an init
 * script re-runs on every load and would resurrect the session behind
 * `wipeClientAuthState`, breaking the logout contract (auth.feature,
 * failure_paths). The write happens once, on a NON-HTML origin document
 * (a static app-shell asset — the WASM app never boots there), so no
 * scenario time is spent booting the app twice. A later wipe therefore
 * removes the session for real, exactly like it removes a UI-login one.
 * The download-consent flag rides the same write (rationale on
 * `addDownloadConsentInitScript`); a wipe still clears it, and the flows
 * that wipe restore it explicitly.
 *
 * Contract: call BEFORE creating scenario pages of this context;
 * navigation and the settle wait stay with the caller
 * (`waitForLoginComplete` is the marker). The login form itself remains
 * under test where it is the behaviour being verified — those callers
 * keep `uiLogin` (auth.feature, the relay scenario of onboarding_scoring)
 * and the manual tools (shots).
 */
export async function apiLogin(
  context: BrowserContext,
  email: string,
  password: string,
  options: { baseURL?: string } = {},
): Promise<void> {
  const baseURL = options.baseURL ?? getAppBaseUrl();
  const { token, refreshToken } = await loginTestUser(email, password);
  const claims = decodeJwtPayload(token);
  const session = {
    auth_token: token,
    refresh_token: refreshToken ?? "",
    email: claims.email ?? "",
    trailbase_id: claims.sub,
    record_id: null,
    expires_at:
      typeof claims.exp === "number" && claims.exp > 0
        ? claims.exp
        : Math.floor(Date.now() / 1000) + 3600,
  };

  // Static, stable, non-HTML asset of the app shell (served next to
  // index.html by both `trunk serve` and the CI `serve dist`): loading it
  // grants same-origin storage access WITHOUT booting the WASM app. The
  // page is throwaway — localStorage outlives it, sessionStorage (any
  // per-page marker) deliberately does not.
  const seeder = await context.newPage();
  try {
    await seeder.goto(`${baseURL}/public/logo-128.png`);
    await seeder.evaluate(
      (seed: { session: object }) => {
        window.localStorage.setItem(
          "origa_resource_download_consented",
          "true",
        );
        window.localStorage.setItem(
          "trailbase_session",
          JSON.stringify(seed.session),
        );
      },
      { session },
    );
  } finally {
    await seeder.close();
  }
}

export async function uiLogin(
  page: Page,
  email: string,
  password: string,
  options: {
    baseURL?: string;
    consent?: boolean;
  } & LoginCompletionOptions = {},
): Promise<LoginCompletionResult> {
  const maxRetries = 3;
  const baseURL = options.baseURL ?? getAppBaseUrl();
  const completion: LoginCompletionOptions = {
    expected: options.expected,
    settleTimeout: options.settleTimeout,
  };

  // The consent seed is shared with apiLogin (the App Review 4.2.3(ii)
  // rationale and the origin guard live on the helper).
  if (options.consent !== false) {
    addDownloadConsentInitScript(page.context(), new URL(baseURL).origin);
  }

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    // The whole login flow is retried, not just waitForURL: the password
    // toggle and the inputs race the WASM cold load too, and a toggle
    // miss used to escape the retry loop as an immediate failure.
    let sessionRetry = false;
    try {
      await page.goto(baseURL, {
        waitUntil: "domcontentloaded",
      });

      // A retried goto boots the app with the session already stored.
      // The login form never mounts there — wait for the terminal state
      // instead of burning another 60 s on the password toggle.
      sessionRetry = attempt > 1 && (await hasStoredSession(page));
      if (sessionRetry) {
        return await waitForLoginComplete(page, {
          ...completion,
          navigations: attempt,
        });
      }

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
      return await waitForLoginComplete(page, completion);
    } catch (e) {
      // An authenticated page has one terminal wait, not three. Surface
      // its diagnostics at once instead of retrying the same dead path.
      if (sessionRetry) {
        throw e;
      }
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

  throw new Error("uiLogin: retry loop finished without a login result");
}
