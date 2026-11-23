import { expect } from "@playwright/test";
import type { BrowserContext, Page } from "@playwright/test";
import { After, Given, Then, When } from "../fixtures";
import { installTauriMock } from "../../helpers/tauri-mock";
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
 * Error-level log messages of the #1482 bug's error branch. Release builds
 * keep `error!` visible (tracing-wasm max level INFO), so a regression that
 * routes foreign URLs back into process_oauth_url trips here.
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
  consoleErrors: string[],
  opts: { currentDeepLink?: string } = {},
): Promise<void> {
  const context = await browser.newContext({ baseURL: getAppBaseUrl() });
  await installTauriMock(context, {
    currentDeepLink: opts.currentDeepLink ?? "",
    pkceVerifier: "e2e-pkce-verifier",
  });
  const page = await context.newPage();
  page.on("console", (msg) => {
    if (msg.type() === "error") consoleErrors.push(msg.text());
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
  async ({ browser, oauthTauriSession, oauthConsoleErrors }) => {
    await bootMockedTauriApp(browser, oauthTauriSession, oauthConsoleErrors);
  },
);

Given(
  "приложение запущено в моке Tauri без сессии с текущим deep-link {string}",
  async ({ browser, oauthTauriSession, oauthConsoleErrors }, url: string) => {
    await bootMockedTauriApp(browser, oauthTauriSession, oauthConsoleErrors, {
      currentDeepLink: url,
    });
  },
);

When(
  "приходит deep-link {string}",
  async ({ oauthTauriSession }, url: string) => {
    const { context } = oauthTauriSession;
    expect(context, "boot step must have run").not.toBeNull();
    const pages = context!.pages();
    await pages[0].evaluate((u) => {
      const mock = (
        window as unknown as {
          __tauriMock?: {
            events: Map<string, Array<(event: { payload: unknown }) => void>>;
          };
        }
      ).__tauriMock;
      if (!mock) throw new Error("__TAURI__ transport mock is not installed");
      for (const handler of mock.events.get("deep-link-received") ?? []) {
        handler({ payload: u });
      }
    }, url);
  },
);

Then(
  "ошибки oauth на странице входа нет",
  async ({ oauthTauriSession, oauthConsoleErrors }) => {
    const { page } = oauthTauriSession;
    expect(page, "boot step must have run").not.toBeNull();
    await page!.waitForTimeout(SETTLE_MS);
    // The user-visible symptom: false «unsupported callback» alert on /login.
    await expect(page!.getByTestId("oauth-error")).toHaveCount(0);
    // The Sentry-noise source: the error branch of the oauth listener must
    // not log at error level for foreign URLs.
    const pinned = oauthConsoleErrors.filter((text) =>
      PINNED_OAUTH_ERROR_LOGS.some((p) => text.includes(p)),
    );
    expect(
      pinned,
      `oauth listener must stay silent for foreign deep-links, got console errors: ${pinned.join(" | ")}`,
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
