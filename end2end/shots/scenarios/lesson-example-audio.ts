import { type Locator, type Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

const CARD_BUDGET = 3;
const RELOAD_BUDGET = 2;
const AUDIO_WAIT_MS = 45_000;

async function visible(locator: Locator, timeout: number): Promise<boolean> {
  try {
    await locator.waitFor({ state: "visible", timeout });
    return true;
  } catch {
    return false;
  }
}

async function advance(page: Page): Promise<boolean> {
  const next = page.getByTestId("lesson-example-next");
  if (await visible(next, 2_000)) {
    await next.click();
    return true;
  }
  const understood = page.getByTestId("lesson-example-understood");
  if (await visible(understood, 2_000)) {
    await understood.click();
    return true;
  }
  return false;
}

/**
 * An example card whose sentence audio button is present. The button
 * mounts after `load_example_detail`, and the `s0000` chunk has a
 * known inflight race, so the walk skips unavailable cards exactly as
 * the covered `lesson-example-recall` scenario does.
 */
export const lessonExampleAudioScenario: ShotScenario = {
  ...descriptors["lesson-example-audio"],
  async run({ page }, sink) {
    for (let attempt = 0; attempt < RELOAD_BUDGET; attempt++) {
      await page.goto("/lesson");
      await page
        .getByTestId("lesson-page")
        .waitFor({ state: "visible", timeout: 90_000 });
      for (let card = 0; card < CARD_BUDGET; card++) {
        if (
          await visible(page.getByTestId("lesson-example-unavailable"), 2_000)
        ) {
          if (!(await advance(page))) break;
          continue;
        }
        if (
          await visible(page.getByTestId("lesson-example-audio"), AUDIO_WAIT_MS)
        ) {
          await page.waitForTimeout(700);
          await sink.shot("example-audio-question");
          return;
        }
        if (!(await advance(page))) break;
      }
      await page.reload({ waitUntil: "domcontentloaded" });
    }
    throw new Error("no example audio button in the bounded card walk");
  },
};
