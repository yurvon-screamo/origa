import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The grammar list page — the rule cards grid.
 *
 * The preset seeds two grammar cards (the acquaintance hand), enough to
 * show the card pattern: one border per card, the rule rendered as card
 * content (task 20261109231401).
 */
export const grammarPageScenario: ShotScenario = {
  ...descriptors["grammar-page"],
  async run({ page }, sink) {
    await page.goto("/grammar");
    try {
      await expect(page.getByTestId("grammar-card-item").first()).toBeVisible({
        timeout: 60_000,
      });
    } catch {
      // A cold page load boots the WASM app from scratch; while the
      // session migration is in flight the protected route shows the
      // login screen even for an authenticated user. The bootstrap
      // persists the user locally, so one reload lands on the page.
      await page.waitForTimeout(25_000);
      await page.reload({ waitUntil: "domcontentloaded" });
      await expect(page.getByTestId("grammar-card-item").first()).toBeVisible({
        timeout: 90_000,
      });
    }
    await page.waitForTimeout(2_000); // fonts settle
    await sink.shot("grid-full", { fullPage: true });
    await sink.shot("grid-viewport");
  },
};
