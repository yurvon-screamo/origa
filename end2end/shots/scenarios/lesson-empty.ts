import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** The lesson page with no due cards and no acquaintance hand. */
export const lessonEmptyScenario: ShotScenario = {
  ...descriptors["lesson-empty"],
  async run({ page }, sink) {
    await page.goto("/lesson");
    await expect(page.getByTestId("lesson-empty-import-btn")).toBeVisible({
      timeout: 60_000,
    });
    await sink.shot("lesson-empty");
  },
};
