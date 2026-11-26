import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** The empty word list of a fresh profile. */
export const listsEmptyScenario: ShotScenario = {
  ...descriptors["lists-empty"],
  async run({ page }, sink) {
    await page.goto("/words");
    await expect(page.getByTestId("words-empty-state")).toBeVisible({
      timeout: 60_000,
    });
    await sink.shot("words-empty");
  },
};
