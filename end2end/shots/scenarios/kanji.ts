import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** The kanji list and its add drawer. */
export const kanjiScenario: ShotScenario = {
  ...descriptors["kanji"],
  async run({ page }, sink) {
    await page.goto("/kanji");
    await expect(page.getByTestId("kanji-page")).toBeVisible({
      timeout: 60_000,
    });
    await sink.shot("kanji-list");
    await page.getByTestId("kanji-add-btn").click();
    await expect(page.getByTestId("kanji-add-drawer")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("kanji-add-modal");
  },
};
