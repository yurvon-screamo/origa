import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The words screen — the vocabulary card grid.
 *
 * The preset seeds exactly one word, enough to show the card pattern:
 * one border per card, the word rendered as card content (task
 * 20261111125434).
 */
export const wordsGridScenario: ShotScenario = {
  ...descriptors["words-grid"],
  async run({ page }, sink) {
    // The `with-word` preset leaves the page on /words after adding the
    // card. A redundant navigation boots the app again and the local
    // seed loses the LWW merge against the server snapshot, so keep the
    // mounted page when the preset already put us here.
    if (!/\/words$/.test(page.url())) await page.goto("/words");
    try {
      await expect(page.getByTestId("words-card-item").first()).toBeVisible({
        timeout: 60_000,
      });
    } catch {
      // A cold page load boots the WASM app from scratch; while the
      // session migration is in flight the protected route shows the
      // login screen even for an authenticated user. The bootstrap
      // persists the user locally, so one reload lands on the page.
      await page.waitForTimeout(25_000);
      await page.reload({ waitUntil: "domcontentloaded" });
      await expect(page.getByTestId("words-card-item").first()).toBeVisible({
        timeout: 90_000,
      });
    }
    await page.waitForTimeout(2_000); // fonts settle
    await sink.shot("grid-full", { fullPage: true });
    await sink.shot("grid-viewport");
  },
};
