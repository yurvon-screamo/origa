import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The words screen (vocabulary list).
 *
 * The header of this screen (title + tab strip + add button) is the
 * narrow-viewport regression surface: the capture pair proves the title
 * and the tabs stay a deliberate grid (title row, full-width tab strip)
 * instead of an accidental wrap.
 */
export const wordsScenario: ShotScenario = {
  ...descriptors.words,
  async run({ page }, sink) {
    await page.goto("/words");
    try {
      await expect(page.getByTestId("words-header")).toBeVisible({
        timeout: 60_000,
      });
    } catch {
      // A cold page load boots the WASM app from scratch; while the
      // session migration is in flight the protected route shows the
      // login screen even for an authenticated user (same workaround
      // as word-examples): one reload lands on the page.
      await page.waitForTimeout(25_000);
      await page.reload({ waitUntil: "domcontentloaded" });
      await expect(page.getByTestId("words-header")).toBeVisible({
        timeout: 90_000,
      });
    }
    await page.waitForTimeout(3_000); // vocabulary chunk fetch
    await sink.shot("words");
    await sink.shot("words-full", { fullPage: true });
  },
};
