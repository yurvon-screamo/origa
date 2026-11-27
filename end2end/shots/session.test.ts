import assert from "node:assert/strict";
import { mock, test } from "node:test";

import type { Browser, BrowserContext, Page } from "@playwright/test";

/**
 * `openShotSession` decides what "login completed" means for a capture.
 * The account helpers, the login helper and the presets are boundaries:
 * only the routing decision (expected terminal + preset) is under test.
 */
type LoginOptions = Record<string, unknown>;

const observed: {
  logins: LoginOptions[];
  presets: string[];
  pages: number;
  closes: number;
} = { logins: [], presets: [], pages: 0, closes: 0 };

function reset(): void {
  observed.logins.length = 0;
  observed.presets.length = 0;
  observed.pages = 0;
  observed.closes = 0;
}

mock.module("../fixtures/admin.ts", {
  namedExports: {
    getAdminToken: async () => ({ token: "token", csrfToken: "csrf" }),
    createTestUser: async () => 1,
    deleteTestUserWithRetry: async () => undefined,
  },
});

mock.module("../helpers/auth.ts", {
  namedExports: {
    DEFAULT_TEST_PASSWORD: "secret",
    generateUniqueEmail: () => "e2e@origa.local",
    uiLogin: async (
      _page: Page,
      _email: string,
      _password: string,
      options: LoginOptions,
    ) => {
      observed.logins.push(options);
      return { elapsedMs: 0 };
    },
  },
});

mock.module("./presets/index.ts", {
  namedExports: {
    presets: {
      fresh: async () => {
        observed.presets.push("fresh");
      },
    },
  },
});

const { openShotSession } = await import("./session.ts");

const fakeBrowser = {
  newContext: async (): Promise<BrowserContext> =>
    ({
      newPage: async (): Promise<Page> => {
        observed.pages += 1;
        return { goto: async () => undefined } as unknown as Page;
      },
      close: async () => {
        observed.closes += 1;
      },
    }) as unknown as BrowserContext,
} as unknown as Browser;

function options(
  overrides: Partial<Parameters<typeof openShotSession>[0]> = {},
): Parameters<typeof openShotSession>[0] {
  return {
    browser: fakeBrowser,
    baseURL: "http://localhost:1421",
    locale: "ru-RU",
    viewport: "desktop",
    preset: "fresh",
    ...overrides,
  };
}

test("consent: required waits for the consent terminal and skips the preset", async () => {
  reset();

  const session = await openShotSession(options({ consent: "required" }));
  await session.close();

  assert.equal(observed.logins.length, 1);
  assert.equal(observed.logins[0]?.expected, "consent");
  assert.equal(observed.logins[0]?.consent, false);
  assert.deepEqual(observed.presets, []);
  assert.equal(observed.pages, 1);
});

test("pre-approved consent waits for the app terminal and applies the preset", async () => {
  reset();

  const session = await openShotSession(options({ consent: "preapproved" }));
  await session.close();

  assert.equal(observed.logins.length, 1);
  assert.equal(observed.logins[0]?.expected, "app");
  assert.equal(observed.logins[0]?.consent, true);
  assert.deepEqual(observed.presets, ["fresh"]);
});

test("an anonymous scenario never calls the login helper", async () => {
  reset();

  const session = await openShotSession(options({ auth: "none" }));
  await session.close();

  assert.deepEqual(observed.logins, []);
  assert.deepEqual(observed.presets, []);
  assert.equal(observed.pages, 1);
});
