import { expect, type Page } from "@playwright/test";

/**
 * Opens a words-family route through the cold-boot flake.
 *
 * A cold page load boots the WASM app from scratch; while the session
 * migration is in flight the protected route shows the login screen even
 * for an authenticated user. The bootstrap persists the user locally, so
 * one reload lands on the page.
 */
export async function gotoWordsRoute(
  page: Page,
  route: string,
  readyTestId = "words-header",
): Promise<void> {
  await page.goto(route);
  try {
    await expect(page.getByTestId(readyTestId)).toBeVisible({
      timeout: 60_000,
    });
  } catch {
    await page.waitForTimeout(25_000);
    await page.reload({ waitUntil: "domcontentloaded" });
    await expect(page.getByTestId(readyTestId)).toBeVisible({
      timeout: 90_000,
    });
  }
}
