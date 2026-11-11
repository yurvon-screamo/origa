import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The yes/no reading-quiz result (task 20261110100449): the ledger must
 * NOT echo the learner's Да/Нет pick (a pass shows the verdict alone, a
 * miss shows only the correct answer) and must offer the full kanji
 * card — readings + meaning visible, the details expand for
 * radicals/writing/examples (QuizFullCardExpand).
 *
 * The `due-corpus` preset makes the kanji cards due (everything else is
 * pushed out), so the lesson body is kanji only. The review view of a
 * kanji is a RANDOM mix (reading quiz / meaning quiz / yesno / writing /
 * normal), so the walk answers every non-yesno card and re-enters the
 * lesson (a reload regenerates every view for free) until a yesno card
 * shows up. A single-select quiz met on the way is captured too — it is
 * the «тест-режим» surface of the same full-card expand.
 */
export const lessonYesnoResultScenario: ShotScenario = {
  ...descriptors["lesson-yesno-result"],
  async run({ page }, sink) {
    let yesnoShot = false;
    let quizShot = false;
    for (let attempt = 0; attempt < 8 && !(yesnoShot && quizShot); attempt++) {
      await page.goto("/lesson");
      await page
        .getByTestId("lesson-page")
        .waitFor({ state: "visible", timeout: 90_000 });
      console.log(`[shots] attempt ${attempt}: walking`);

      for (let card = 0; card < 14; card++) {
        // The target: the yes/no card → result without the pick echo,
        // then the expanded full kanji card.
        const yesno = page.getByTestId("yesno-no-btn");
        if ((await yesno.isVisible().catch(() => false)) && !yesnoShot) {
          await yesno.click({ timeout: 4_000 }).catch(() => undefined);
          await page.waitForTimeout(600);
          await sink.shot("yesno-result");
          const details = page.getByTestId("kanji-details-toggle");
          if (await details.isVisible().catch(() => false)) {
            await details.click({ timeout: 4_000 }).catch(() => undefined);
            await page.waitForTimeout(700);
            await sink.shot("yesno-result-expanded");
          }
          yesnoShot = true;
          await page
            .getByTestId("lesson-card-next-btn")
            .click({ timeout: 4_000 })
            .catch(() => undefined);
          await page.waitForTimeout(400);
          continue;
        }

        // Multi-select quiz first: the submit button is the multi
        // discriminator — quiz-option-N test ids exist on BOTH variants.
        const submit = page.getByTestId("quiz-submit-btn");
        if (await submit.isVisible().catch(() => false)) {
          const first = page.getByTestId("quiz-option-0");
          if (await first.isVisible().catch(() => false)) {
            await first.click({ timeout: 4_000 }).catch(() => undefined);
            await page.waitForTimeout(300);
          }
          await submit.click({ timeout: 4_000 }).catch(() => undefined);
          await page.waitForTimeout(600);
          await page
            .getByTestId("lesson-card-next-btn")
            .click({ timeout: 4_000 })
            .catch(() => undefined);
          await page.waitForTimeout(400);
          continue;
        }

        // Opportunistic: the single-select quiz (Тест) — the second
        // surface of the full-card expand. Reached only when the multi
        // submit button is absent (the discriminator above).
        const singleOption = page.getByTestId("quiz-option-0");
        if ((await singleOption.isVisible().catch(() => false)) && !quizShot) {
          await singleOption.click({ timeout: 4_000 }).catch(() => undefined);
          await page.waitForTimeout(600);
          await sink.shot("quiz-result");
          const details = page.getByTestId("kanji-details-toggle");
          if (await details.isVisible().catch(() => false)) {
            await details.click({ timeout: 4_000 }).catch(() => undefined);
            await page.waitForTimeout(700);
            await sink.shot("quiz-result-expanded");
          }
          quizShot = true;
          await page
            .getByTestId("lesson-card-next-btn")
            .click({ timeout: 4_000 })
            .catch(() => undefined);
          await page.waitForTimeout(400);
          continue;
        }

        // Normal card: reveal, rate «хорошо».
        const reveal = page.getByTestId("lesson-show-answer-btn");
        if (await reveal.isVisible().catch(() => false)) {
          await reveal.click({ timeout: 4_000 }).catch(() => undefined);
          await page.waitForTimeout(600);
          await page
            .getByTestId("lesson-rating-btn-again")
            .click({ timeout: 4_000 })
            .catch(() => undefined);
          await page.waitForTimeout(400);
          continue;
        }

        // Held on the feedback card — advance.
        const nextBtn = page.getByTestId("lesson-card-next-btn");
        if (await nextBtn.isVisible().catch(() => false)) {
          await nextBtn.click({ timeout: 4_000 }).catch(() => undefined);
          await page.waitForTimeout(400);
          continue;
        }

        // The writing card (canvas drawing, no skip) — abandon the
        // attempt; the outer loop re-enters the lesson.
        if (
          await page
            .locator("canvas")
            .isVisible()
            .catch(() => false)
        ) {
          console.log(
            `[shots] attempt ${attempt} card ${card}: writing card, reroll`,
          );
          break;
        }

        await page.waitForTimeout(700);
      }
    }

    expect(
      yesnoShot,
      "a yes/no card must appear within the reroll budget",
    ).toBe(true);
    if (!quizShot) {
      console.log(
        "[shots] no single-select quiz met on the way — the тест-режим shot is skipped",
      );
    }
  },
};
