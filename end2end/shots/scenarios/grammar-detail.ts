import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * A grammar rule detail: overview, the practice session and the delete
 * confirmation. The practice preset seeds a quiz-capable rule plus one
 * known verb — the same pair the BDD grammar steps use, without the
 * minutes of UI seeding on every viewport.
 */
export const grammarDetailScenario: ShotScenario = {
  ...descriptors["grammar-detail"],
  async run({ page }, sink) {
    await page.goto("/grammar");
    await expect(page.getByTestId("grammar-card-item").first()).toBeVisible({
      timeout: 60_000,
    });
    await page.getByTestId("grammar-card-item").first().click();
    await expect(page.getByTestId("grammar-detail-container")).toBeVisible({
      timeout: 30_000,
    });
    await sink.shot("grammar-detail-overview");

    const practiceTab = page.getByTestId("grammar-detail-tabs-practice");
    if (await practiceTab.isVisible().catch(() => false)) {
      await practiceTab.click();
    }
    await expect(
      page
        .getByTestId("grammar-practice-progress")
        .or(page.getByTestId("grammar-practice-complete"))
        .or(page.getByTestId("grammar-practice-no-words")),
    ).toBeVisible({ timeout: 15_000 });
    if (
      await page
        .getByTestId("grammar-practice-complete")
        .isVisible()
        .catch(() => false)
    ) {
      await page.getByTestId("grammar-practice-again-btn").click();
    }
    await expect(page.getByTestId("grammar-practice-progress")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("grammar-practice-question");

    const deleteButton = page
      .getByTestId("grammar-detail-actions-delete-btn")
      .or(page.getByTestId("grammar-detail-actions-mobile-delete-btn"));
    await deleteButton.first().click();
    await expect(page.getByTestId("grammar-detail-delete-modal")).toBeVisible({
      timeout: 10_000,
    });
    await sink.shot("grammar-delete-modal");
  },
};
