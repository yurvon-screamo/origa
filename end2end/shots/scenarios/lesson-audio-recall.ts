import { type Locator } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { walkLessonCards } from "./lesson-view-walk.ts";

const CARD_BUDGET = 30;
const CONTROL_TIMEOUT = 4_000;

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
    const found = await walkLessonCards(
      page,
      async (walkPage) => {
        if (
          !(await visible(
            walkPage.getByTestId("audio-recall-card-root"),
            2_000,
          ))
        ) {
          return false;
        }
        await walkPage.waitForTimeout(700);
        await sink.shot("audio-recall-question");
        await walkPage.getByTestId("audio-recall-reveal-btn").click();
        if (
          !(await visible(
            walkPage.getByTestId("audio-recall-know-btn"),
            10_000,
          ))
        ) {
          throw new Error("audio-recall answer side never appeared");
        }
        await sink.shot("audio-recall-answer");
        return true;
      },
      CARD_BUDGET,
      CONTROL_TIMEOUT,
    );
    if (!found) {
      throw new Error("no audio-recall view in the bounded card walk");
    }
  },
};
