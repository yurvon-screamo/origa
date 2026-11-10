import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The counters detail page — readings table and the examples section.
 *
 * The preset seeds one counter card (本, the counter-due preset); the
 * detail shows the hero, the readings table and the examples card as
 * top-level sections — one border per card (task 20261109231401).
 */
export const countersDetailScenario: ShotScenario = {
  ...descriptors["counters-detail"],
  async run({ page }, sink) {
    await page.goto("/counters");
    const card = page
      .locator('[data-testid="counter-card-item"]', { hasText: "本" })
      .first();
    try {
      await expect(card).toBeVisible({ timeout: 60_000 });
    } catch {
      // A cold page load boots the WASM app from scratch; while the
      // session migration is in flight the protected route shows the
      // login screen even for an authenticated user. The bootstrap
      // persists the user locally, so one reload lands on the page.
      await page.waitForTimeout(25_000);
      await page.reload({ waitUntil: "domcontentloaded" });
      await expect(card).toBeVisible({ timeout: 90_000 });
    }
    await card.click();
    await expect(page.getByTestId("counters-detail-hero")).toBeVisible({
      timeout: 30_000,
    });
    // The examples card renders only for suffixes with dictionary
    // words; 本 has plenty — its presence is part of the screen.
    await expect(page.getByTestId("counters-detail-examples")).toBeVisible({
      timeout: 30_000,
    });
    await page.waitForTimeout(2_000); // fonts settle
    await sink.shot("detail-full", { fullPage: true });
    await sink.shot("detail-viewport");
  },
};
