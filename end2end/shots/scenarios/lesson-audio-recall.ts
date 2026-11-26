import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** An audio-recall card: question and answered side. */
export const lessonAudioRecallScenario: ShotScenario = {
  ...descriptors["lesson-audio-recall"],
  async run({ page }, sink) {
    for (let attempt = 0; attempt < 8; attempt++) {
      await page.goto("/lesson");
      const card = page.getByTestId("audio-recall-card-root");
      if (
        await card
          .waitFor({ state: "visible", timeout: 90_000 })
          .then(() => true)
          .catch(() => false)
      ) {
        await page.waitForTimeout(700);
        await sink.shot("audio-recall-question");
        await page.getByTestId("audio-recall-reveal-btn").click();
        await expect(page.getByTestId("audio-recall-know-btn")).toBeVisible({
          timeout: 10_000,
        });
        await sink.shot("audio-recall-answer");
        return;
      }
      await page.reload({ waitUntil: "domcontentloaded" });
    }
    throw new Error("no audio-recall view in the bounded reroll budget");
  },
};
