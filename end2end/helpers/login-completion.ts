import type { Page } from "@playwright/test";

export type LoginTerminal = "app" | "consent";

export interface LoginCompletionOptions {
  /** Which state counts as a completed login. BDD callers omit this. */
  expected?: LoginTerminal;
  /** Budget for the terminal state after a submit. */
  settleTimeout?: number;
  /** First window before a reload is considered in shots. */
  stallTimeout?: number;
  /** Allow one reload with the stored session when the first window stalls. */
  recoverOnStall?: boolean;
}

export interface LoginCompletionResult {
  restarted: boolean;
  elapsedMs: number;
}

const DEFAULT_SETTLE_TIMEOUT = 90_000;
const DEFAULT_STALL_TIMEOUT = 30_000;
const DEFAULT_URL_TIMEOUT = 90_000;

function terminalLocator(page: Page, expected: LoginTerminal) {
  if (expected === "consent") {
    return page.getByTestId("resource-consent-download-button");
  }
  return page
    .getByTestId("home-content")
    .or(page.getByTestId("onboarding-stepper"));
}

async function waitTerminal(
  page: Page,
  expected: LoginTerminal,
  timeout: number,
): Promise<void> {
  await terminalLocator(page, expected).waitFor({ state: "visible", timeout });
}

async function snapshot(page: Page, expected: LoginTerminal): Promise<string> {
  const state = await page.evaluate(() => ({
    url: location.pathname,
    session: Boolean(localStorage.getItem("trailbase_session")),
    consent: Boolean(
      document.querySelector(
        '[data-testid="resource-consent-download-button"]',
      ),
    ),
    wizard: Boolean(
      document.querySelector('[data-testid="onboarding-stepper"]'),
    ),
    content: Boolean(document.querySelector('[data-testid="home-content"]')),
    toggle: Boolean(
      document.querySelector('[data-testid="login-password-toggle"]'),
    ),
  }));
  return `terminal=${expected} url=${state.url} session=${state.session} consent=${state.consent} wizard=${state.wizard} content=${state.content} toggle=${state.toggle}`;
}

/**
 * Waits for the state that proves a login completed.
 *
 * The default keeps the BDD contract: terminal URL first, then the old
 * root-route fallback. Shots pass `expected` to skip the URL wait — the
 * consent screen never matches a terminal URL, and a fresh profile can
 * render Home before the onboarding guard moves it to the wizard.
 */
export async function waitForLoginComplete(
  page: Page,
  options: LoginCompletionOptions = {},
): Promise<LoginCompletionResult> {
  const startedAt = Date.now();

  if (!options.expected) {
    try {
      await page.waitForURL(/\/(home|onboarding)$/, {
        timeout: DEFAULT_URL_TIMEOUT,
      });
    } catch {
      await page.getByTestId("login-password-toggle").waitFor({
        state: "hidden",
        timeout: 15_000,
      });
      await page
        .getByTestId("home-page")
        .or(page.getByTestId("onboarding-stepper"))
        .waitFor({ state: "visible", timeout: 60_000 });
    }
    return { restarted: false, elapsedMs: Date.now() - startedAt };
  }

  const expected = options.expected;
  const settleTimeout = options.settleTimeout ?? DEFAULT_SETTLE_TIMEOUT;

  try {
    if (options.recoverOnStall && expected === "app") {
      const stallTimeout = options.stallTimeout ?? DEFAULT_STALL_TIMEOUT;
      try {
        await waitTerminal(page, expected, stallTimeout);
      } catch {
        await page.reload();
        await waitTerminal(page, expected, settleTimeout);
        return { restarted: true, elapsedMs: Date.now() - startedAt };
      }
    }
    await waitTerminal(page, expected, settleTimeout);
  } catch (error) {
    throw new Error(
      `Login did not reach ${expected} terminal: ${await snapshot(page, expected)}`,
      { cause: error },
    );
  }

  return { restarted: false, elapsedMs: Date.now() - startedAt };
}

/** True when a previous submit already left a session in localStorage. */
export async function hasStoredSession(page: Page): Promise<boolean> {
  return page.evaluate(() =>
    Boolean(localStorage.getItem("trailbase_session")),
  );
}
