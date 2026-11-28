import { type Locator } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { ADVANCES, REVEALS } from "./lesson-card-controls.ts";
import { clickFirstVisible } from "./review-body.ts";

const CARD_BUDGET = 30;

async function visible(locator: Locator, timeout: number): Promise<boolean> {
  try {
    await locator.waitFor({ state: "visible", timeout });
    return true;
  } catch {
    return false;
  }
}

/** An audio-recall card: question and answered side. */
export const lessonAudioRecallScenario: ShotScenario = {
  ...descriptors["lesson-audio-recall"],
  async run({ page }, sink) {
    await page.goto("/lesson");
    await page
      .locator("#origa-boot-splash")
      .waitFor({ state: "detached", timeout: 180_000 })
      .catch(() => undefined);
    for (let card = 0; card < CARD_BUDGET; card++) {
      if (await visible(page.getByTestId("audio-recall-card-root"), 2_000)) {
        await page.waitForTimeout(700);
        await sink.shot("audio-recall-question");
        await page.getByTestId("audio-recall-reveal-btn").click();
        if (!(await visible(page.getByTestId("audio-recall-know-btn"), 10_000))) {
          throw new Error("audio-recall answer side never appeared");
        }
        await sink.shot("audio-recall-answer");
        return;
      }
      if (await page.getByTestId("lesson-complete-screen").isVisible()) {
        break;
      }
      const advanced =
        (await clickFirstVisible(page, [...REVEALS, ...ADVANCES], 4_000)) !==
        null;
      if (!advanced) break;
    }
    throw new Error("no audio-recall view in the bounded card walk");
  },
};
