import assert from "node:assert/strict";
import { mock, test } from "node:test";

import type { Page } from "@playwright/test";

/**
 * The toast wait is the boundary between a deterministic baseline and a
 * false diff. Playwright itself is the external dependency here: the
 * helper's decisions — give the toast a window to mount, then insist it
 * is gone before the screenshot — are the behaviour under test.
 */
const calls: string[] = [];
let toastMounted = true;
let toastStuck = false;

mock.module("@playwright/test", {
  namedExports: {
    expect: () => ({
      toHaveCount: async (
        expected: number,
        options: { timeout: number },
      ): Promise<void> => {
        calls.push(`count:${expected}:${options.timeout}`);
        if (toastStuck) throw new Error("toast still visible");
      },
    }),
  },
});

const { waitForSyncToastToDisappear } = await import("./simple-page.ts");

function fakePage(): Page {
  const locator = {
    first: () => locator,
    waitFor: async (options: {
      state: string;
      timeout: number;
    }): Promise<void> => {
      calls.push(`wait:${options.state}:${options.timeout}`);
      if (options.state === "visible" && !toastMounted) {
        throw new Error("no toast appeared within the mount window");
      }
    },
  };
  return {
    locator: (selector: string) => {
      calls.push(`locator:${selector}`);
      return locator;
    },
  } as unknown as Page;
}

function reset(toast: boolean, stuck = false): void {
  calls.length = 0;
  toastMounted = toast;
  toastStuck = stuck;
}

test("the wait gives the toast a mount window before asserting it vanished", async () => {
  reset(true);

  await waitForSyncToastToDisappear(fakePage());

  assert.ok(
    calls.includes('locator:[data-testid^="toast-"]'),
    "the toast locator was not queried",
  );
  const mount = calls.findIndex((call) => call.startsWith("wait:visible"));
  const vanish = calls.findIndex((call) => call.startsWith("count:0:"));
  assert.ok(mount >= 0, "the toast was not given a mount window");
  assert.ok(vanish >= 0, "the toast disappearance was not asserted");
  assert.ok(mount < vanish, "the mount wait must precede the vanish check");
});

test("a page with no toast still passes the wait", async () => {
  reset(false);

  await waitForSyncToastToDisappear(fakePage());

  assert.ok(
    calls.some((call) => call.startsWith("count:0:")),
    "the disappearance must still be asserted without a toast",
  );
});

test("a toast that never leaves fails the wait", async () => {
  reset(true, true);

  await assert.rejects(
    waitForSyncToastToDisappear(fakePage()),
    /toast still visible/,
  );
});
