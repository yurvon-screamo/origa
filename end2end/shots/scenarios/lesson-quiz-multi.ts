import { expect, type Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { walkLessonCards } from "./lesson-view-walk.ts";

/**
 * A multi-choice quiz: two selected tiles stay selected at once. A
 * single-choice quiz clears the first tile on the second click, so the
 * selection count is the DOM discriminator (the options carry no view
 * test-id of their own).
 */
export const lessonQuizMultiScenario: ShotScenario = {
  ...descriptors["lesson-quiz-multi"],
  async run({ page }, sink) {
    for (let attempt = 0; attempt < 3; attempt++) {
      await page.goto("/lesson");
      await page
        .getByTestId("lesson-page")
        .waitFor({ state: "visible", timeout: 90_000 });
      const found = await walkLessonCards(page, async () => {
        const first = page.getByTestId("quiz-option-0");
        const quizVisible = await first
          .waitFor({ state: "visible", timeout: 1_000 })
          .then(() => true)
          .catch(() => false);
        if (!quizVisible) return false;
        await first.click();
        await page.getByTestId("quiz-option-1").click();
        if ((await selectedOptions(page)) < 2) return false;
        await page.waitForTimeout(700);
        await sink.shot("multi-quiz-question", { marker: "quiz-option-1" });
        await page.getByTestId("quiz-submit-btn").click();
        await expect(page.getByTestId("quiz-feedback-strip")).toBeVisible({
          timeout: 10_000,
        });
        await sink.shot("multi-quiz-result", {
          marker: "quiz-feedback-strip",
        });
        return true;
      });
      if (found) return;
    }
    throw new Error("no multi-choice quiz in the lesson card walk");
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
