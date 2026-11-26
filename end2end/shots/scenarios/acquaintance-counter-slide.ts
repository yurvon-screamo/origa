import { expect, type Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** The counter slide of the acquaintance hand: presentation and answer. */
export const acquaintanceCounterSlideScenario: ShotScenario = {
  ...descriptors["acquaintance-counter-slide"],
  async run({ page }, sink) {
    await page.goto("/lesson");
    await page
      .getByTestId("acquaintance-view")
      .waitFor({ state: "visible", timeout: 180_000 });

    const next = page.getByTestId("acquaintance-next-btn");
    let presentation = false;
    for (let index = 0; index < 12 && !presentation; index++) {
      if (
        await page
          .getByTestId("acquaintance-counter-slide")
          .isVisible()
          .catch(() => false)
      ) {
        await page.waitForTimeout(700);
        await sink.shot("counter-presentation");
        presentation = true;
        break;
      }
      await next.click({ timeout: 2_000 }).catch(() => undefined);
      await page.waitForTimeout(500);
    }
    expect(presentation, "the hand must show the counter slide").toBe(true);

    let answer = false;
    for (let index = 0; index < 60 && !answer; index++) {
      if (await finished(page)) break;
      if (
        !(await page
          .getByTestId("acquaintance-training")
          .isVisible()
          .catch(() => false))
      ) {
        await page.waitForTimeout(800);
        continue;
      }
      const reveal = page.getByTestId("acquaintance-reveal-btn");
      if (await reveal.isVisible().catch(() => false)) {
        await reveal.click({ timeout: 3_000 }).catch(() => undefined);
        if (
          await page
            .getByTestId("acquaintance-counter-answer")
            .waitFor({ state: "visible", timeout: 4_000 })
            .then(() => true)
            .catch(() => false)
        ) {
          await page.waitForTimeout(700);
          await sink.shot("counter-training-answer");
          answer = true;
          break;
        }
      }
      await page
        .getByTestId("acquaintance-rating-remember")
        .click({ timeout: 2_000 })
        .catch(() => undefined);
      await page.waitForTimeout(500);
    }
    expect(answer, "the hand must show the counter answer").toBe(true);
  },
};

function finished(page: Page): Promise<boolean> {
  return page
    .getByTestId("acquaintance-completed")
    .or(page.getByTestId("lesson-complete-screen"))
    .isVisible()
    .catch(() => false);
}
