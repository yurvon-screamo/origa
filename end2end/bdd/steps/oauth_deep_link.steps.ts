import { expect } from "@playwright/test";
import type { BrowserContext, Page } from "@playwright/test";
import { After, Given, Then, When } from "../fixtures";
import { dispatchDeepLink, installTauriMock } from "../../helpers/tauri-mock";
import { getAppBaseUrl } from "../../config";

/**
 * OAuth deep-link routing (#1482). The scenarios boot the app inside a
 * `window.__TAURI__` transport mock (helpers/tauri-mock.ts, shape from
 * core/tauri.rs) as an unauthenticated user on /login — the exact state of
 * the bug: a foreign deep-link URL (selftest probe, word-adding shortcut)
 * used to fall into process_oauth_url's error branch, producing a Sentry
 * error-event and a false «unsupported callback» alert on /login.
 *
 * The positive control observes the happy path at the network boundary:
 * a real `origa://auth/callback?code=…` must fire the token-exchange POST
 * (debug logs are invisible in the release dist CI runs against, but the
 * fetch is not).
 */

/** Settle window for the async spawn_local processing after an event. */
const SETTLE_MS = 1500;

/**
 * Error-branch log messages of the #1482 bug. tracing-wasm emits EVERY
 * tracing level through console.log (the crate ships no console.error
 * binding), so the console capture is type-agnostic and pins by substring:
 * `error!` stays visible in both build profiles (max level DEBUG in dev,
 * INFO in release — origa_ui/src/lib.rs), so a regression that routes
 * foreign URLs back into process_oauth_url trips here on console.log lines.
 */
const PINNED_OAUTH_ERROR_LOGS = [
  "URL has no 'code' param and no fragment",
  "OAuth callback error",
  "invalid deep-link event format",
];

interface OAuthTauriSession {
  page: Page | null;
  context: BrowserContext | null;
  tokenRequests: string[];
}

async function bootMockedTauriApp(
  browser: import("@playwright/test").Browser,
  session: OAuthTauriSession,
  consoleLogs: string[],
  opts: { currentDeepLink?: string } = {},
): Promise<void> {
  const context = await browser.newContext({ baseURL: getAppBaseUrl() });
  // Register for teardown before anything can fail: if boot dies below
  // (e.g. the settle marker times out), the After hook still closes the
  // context instead of leaking it.
  session.context = context;
  await installTauriMock(context, {
    currentDeepLink: opts.currentDeepLink ?? "",
    pkceVerifier: "e2e-pkce-verifier",
  });
  const page = await context.newPage();
  // All console types: the pinned error-branch messages arrive as
  // console.log (tracing-wasm), app panics as console.error — pinning is
  // by substring, the type prefix keeps failure output diagnosable.
  page.on("console", (msg) => {
    consoleLogs.push(`[${msg.type()}] ${msg.text()}`);
  });
  page.on("request", (req) => {
    if (req.url().includes("/api/auth/v1/token"))
      session.tokenRequests.push(req.url());
  });
  await page.goto(`${getAppBaseUrl()}/login`, {
    waitUntil: "domcontentloaded",
  });
  // Settle marker: the login card mounted means the app booted and the
  // global oauth listener (app.rs) had its chance to register + cold-poll.
  await expect(page.getByTestId("login-card")).toBeVisible({ timeout: 60_000 });
  session.context = context;
  session.page = page;
}

Given(
  "приложение запущено в моке Tauri без сессии",
  async ({ browser, oauthTauriSession, oauthConsoleLogs }) => {
    await bootMockedTauriApp(browser, oauthTauriSession, oauthConsoleLogs);
  },
);

Given(
  "приложение запущено в моке Tauri без сессии с текущим deep-link {string}",
  async ({ browser, oauthTauriSession, oauthConsoleLogs }, url: string) => {
    await bootMockedTauriApp(browser, oauthTauriSession, oauthConsoleLogs, {
      currentDeepLink: url,
    });
  },
);

When(
  "приходит deep-link {string}",
  async ({ oauthTauriSession }, url: string) => {
    const { context } = oauthTauriSession;
    expect(context, "boot step must have run").not.toBeNull();
    await dispatchDeepLink(context!, url);
  },
);

Then(
  "ошибки oauth на странице входа нет",
  async ({ oauthTauriSession, oauthConsoleLogs }) => {
    const { page } = oauthTauriSession;
    expect(page, "boot step must have run").not.toBeNull();
    await page!.waitForTimeout(SETTLE_MS);
    // The user-visible symptom: false «unsupported callback» alert on /login.
    await expect(page!.getByTestId("oauth-error")).toHaveCount(0);
    // The Sentry-noise source: the error branch of the oauth listener must
    // not log at error level for foreign URLs. The capture is type-agnostic
    // (see PINNED_OAUTH_ERROR_LOGS) — the pin reads console.log lines too.
    const pinned = oauthConsoleLogs.filter((text) =>
      PINNED_OAUTH_ERROR_LOGS.some((p) => text.includes(p)),
    );
    expect(
      pinned,
      `oauth listener must stay silent for foreign deep-links, got pinned console lines: ${pinned.join(" | ")}`,
    ).toEqual([]);
  },
);

Then("запрос обмена кода уходит в TrailBase", async ({ oauthTauriSession }) => {
  const { tokenRequests } = oauthTauriSession;
  await expect
    .poll(() => tokenRequests.length, { timeout: 15_000 })
    .toBeGreaterThan(0);
  expect(
    tokenRequests.some((url) => url.includes("/api/auth/v1/token")),
    `token exchange request expected, got: ${tokenRequests.join(", ")}`,
  ).toBe(true);
});

After(async ({ oauthTauriSession }) => {
  await oauthTauriSession.context?.close();
});
