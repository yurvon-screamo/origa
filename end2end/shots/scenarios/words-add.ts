import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** The add-words drawer and its input modes. */
export const wordsAddScenario: ShotScenario = {
  ...descriptors["words-add"],
  async run({ page }, sink) {
    await page.goto("/words");
    await expect(page.getByTestId("words-add-btn")).toBeVisible({
      timeout: 60_000,
    });
    await page.getByTestId("words-add-btn").click();
    await expect(page.getByTestId("words-add-drawer")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("words-add-input", { marker: "words-add-drawer" });

    await page.getByTestId("words-drawer-textarea").fill("本");
    await page.getByTestId("words-drawer-analyze-btn").click();
    await expect(page.getByTestId("words-drawer-item").first()).toBeVisible({
      timeout: 30_000,
    });
    await sink.shot("words-add-preview", { marker: "words-drawer-item" });

    for (const [tab, screen] of [
      ["anki", "words-add-anki"],
      ["image", "words-add-image"],
      ["audio", "words-add-audio"],
    ] as const) {
      await page.getByTestId(`words-add-tabs-${tab}`).click();
      await expect(page.getByTestId("words-add-drawer")).toBeVisible();
      await sink.shot(screen, { marker: "words-add-drawer" });
    }
  },
};
