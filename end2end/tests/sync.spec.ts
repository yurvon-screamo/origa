import { testWithFreshUser } from "../fixtures";
import { skipOnboarding } from "../helpers/navigation";
import { setupTestUser, uiLogin } from "../helpers/auth";
import { loginTestUser } from "../fixtures/admin";
import { fetchWithTimeout } from "../helpers/http";
import { getTrailBaseUrl } from "../config";
import { expect, test, type Page } from "@playwright/test";

const NIL_USER_ID = "0".repeat(26);

/**
 * Reads all IndexedDB keys from the "users" object store. Rejects on database
 * errors so tests cannot pass for the wrong reason (e.g. empty result masking
 * a real failure to open the store).
 */
async function readUserStoreKeys(page: Page): Promise<string[]> {
  return await page.evaluate(async (): Promise<string[]> => {
    return await new Promise<string[]>((resolve, reject) => {
      const open = indexedDB.open("origa");
      open.onerror = () => reject(new Error("indexedDB open failed"));
      open.onupgradeneeded = () => resolve([]);
      open.onsuccess = () => {
        const db = open.result;
        if (!db.objectStoreNames.contains("users")) {
          resolve([]);
          return;
        }
        const tx = db.transaction("users", "readonly");
        const store = tx.objectStore("users");
        const req = store.getAllKeys();
        req.onsuccess = () => resolve(req.result.map((k) => String(k)));
        req.onerror = () => reject(new Error("getAllKeys failed"));
      };
    });
  });
}

/**
 * Polls IndexedDB until a non-nil user key appears (the canonical record the
 * app writes after a successful sync) or the timeout elapses. Replaces a fixed
 * sleep with a deterministic readiness signal.
 */
async function waitForCanonicalUserKey(
  page: Page,
  timeoutMs = 10_000,
): Promise<string | null> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const keys = await readUserStoreKeys(page).catch(() => [] as string[]);
    const canonical = keys.find((k) => !k.endsWith(`:${NIL_USER_ID}`));
    if (canonical) return canonical;
    await page.waitForTimeout(250);
  }
  return null;
}

/**
 * Counts `user` records owned by the authenticated user whose email matches.
 * The records API uses a filter expression; we query the canonical table that
 * the app syncs to.
 */
async function countRemoteUserRecords(
  token: string,
  email: string,
): Promise<number> {
  const url = `${getTrailBaseUrl()}/api/records/v1/user?filter[email][$eq]=${encodeURIComponent(
    email,
  )}`;
  const response = await fetchWithTimeout(url, {
    method: "GET",
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!response.ok) {
    throw new Error(
      `records list failed: ${response.status} ${await response.text()}`,
    );
  }
  const data = (await response.json()) as { records: unknown[] };
  return data.records.length;
}

testWithFreshUser.describe("User Identity Sync", () => {
  testWithFreshUser(
    "user id is never nil after login @critical",
    async ({ page }) => {
      const nilLogHits: string[] = [];
      page.on("console", (msg) => {
        const text = msg.text();
        if (text.includes(NIL_USER_ID)) {
          nilLogHits.push(text);
        }
      });

      await skipOnboarding(page);
      await page.waitForURL(/\/home$/, { timeout: 15_000 });
      await page
        .getByTestId("home-page")
        .waitFor({ state: "visible", timeout: 30_000 });

      // Deterministic readiness signal: the canonical (non-nil) user key
      // must appear in IndexedDB after the synced save flushes.
      const canonicalKey = await waitForCanonicalUserKey(page);
      expect(
        canonicalKey,
        "Expected a non-nil user key in IndexedDB after login",
      ).not.toBeNull();

      const keys = await readUserStoreKeys(page);
      const nilKeys = keys.filter((k) => k.endsWith(`:${NIL_USER_ID}`));

      expect(
        nilKeys,
        `IndexedDB must not hold a nil user key; found: ${nilKeys.join(", ")}`,
      ).toHaveLength(0);
      expect(
        nilLogHits,
        `save must not be attributed to the nil user id; saw: ${nilLogHits.join(" | ")}`,
      ).toHaveLength(0);
    },
  );
});

test.describe("Multi-browser sync @critical", () => {
  // NOTE: this test runs browsers A and B sequentially, so it catches the
  // duplicate-creation regression where a single browser somehow writes two
  // remote rows. It does NOT cover the harder concurrent-first-login race,
  // where A and B both call `find_current → None` simultaneously and both
  // `create` — that is a server-side read-modify-write race in the upsert
  // path and is out of scope for this client-side fix.
  test("two browsers on the same account share one remote user record", async ({
    browser,
  }) => {
    const userCtx = await setupTestUser();
    try {
      const ctxA = await browser.newContext();
      const pageA = await ctxA.newPage();
      await pageA.setViewportSize({ width: 1280, height: 720 });
      await uiLogin(pageA, userCtx.email, userCtx.password);
      await skipOnboarding(pageA);
      await pageA
        .getByTestId("home-page")
        .waitFor({ state: "visible", timeout: 30_000 });

      // First browser has flushed a remote record through the synced
      // save path; there must be exactly one for this email.
      const { token } = await loginTestUser(userCtx.email, userCtx.password);
      const countAfterA = await countRemoteUserRecords(token, userCtx.email);
      // Strict equality: the first browser must produce exactly one
      // remote record for this account. `toBeGreaterThanOrEqual(1)`
      // would silently pass even if browser A created two rows (the
      // duplicate-creation regression this test exists to catch).
      expect(countAfterA).toBe(1);

      const ctxB = await browser.newContext();
      const pageB = await ctxB.newPage();
      await pageB.setViewportSize({ width: 1280, height: 720 });
      await uiLogin(pageB, userCtx.email, userCtx.password);
      await skipOnboarding(pageB);
      await pageB
        .getByTestId("home-page")
        .waitFor({ state: "visible", timeout: 30_000 });

      // Second browser must reuse the existing remote record, not
      // create a duplicate. A duplicate would split progress again.
      const countAfterB = await countRemoteUserRecords(token, userCtx.email);
      expect(countAfterB).toBe(countAfterA);

      await ctxA.close();
      await ctxB.close();
    } finally {
      await userCtx.cleanup();
    }
  });
});
