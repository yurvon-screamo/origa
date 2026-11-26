import { expect, type Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * A multi-choice quiz: two selected tiles stay selected at once. A
 * single-choice quiz clears the first tile on the second click, so the
 * selection count is the DOM discriminator (the options carry no view
 * test-id of their own).
 */
export const lessonQuizMultiScenario: ShotScenario = {
  ...descriptors["lesson-quiz-multi"],
  async run({ page }, sink) {
    for (let attempt = 0; attempt < 8; attempt++) {
      await page.goto("/lesson");
      const first = page.getByTestId("quiz-option-0");
      const second = page.getByTestId("quiz-option-1");
      if (
        !(await first
          .waitFor({ state: "visible", timeout: 90_000 })
          .then(() => true)
          .catch(() => false))
      ) {
        await page.reload({ waitUntil: "domcontentloaded" });
        continue;
      }
      await first.click();
      await second.click();
      if ((await selectedOptions(page)) < 2) {
        await page.reload({ waitUntil: "domcontentloaded" });
        continue;
      }
      await page.waitForTimeout(700);
      await sink.shot("multi-quiz-question");
      await page.getByTestId("quiz-submit-btn").click();
      await expect(page.getByTestId("quiz-feedback-strip")).toBeVisible({
        timeout: 10_000,
      });
      await sink.shot("multi-quiz-result");
      return;
    }
    throw new Error("no multi-choice quiz in the bounded reroll budget");
  },
};

async function selectedOptions(page: Page): Promise<number> {
  return await page
    .locator('[data-testid^="quiz-option-"]')
    .evaluateAll(
      (elements) =>
        elements.filter((element) => element.className.includes("accent-olive"))
          .length,
    );
}
