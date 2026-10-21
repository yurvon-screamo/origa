import { expect } from "@playwright/test";

import { HomePage } from "../../pages/home.page.ts";
import { LessonPage } from "../../pages/lesson.page.ts";
import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The lesson in its two shapes: the acquaintance slide and the answer side
 * of the review body.
 */
export const lessonExampleScenario: ShotScenario = {
    ...descriptors["lesson-example"],
    async run({ page }, sink) {
        const home = new HomePage(page);
        const wordsNav = page.getByRole("link", { name: "Words" });
        await wordsNav.waitFor({ state: "visible", timeout: 600_000 });
        await home.startLesson();

        const lesson = new LessonPage(page);
        await lesson.expectLessonVisible();

        let acquaintanceShot = false;
        let answerShot = false;
        try {
            await expect(page.getByTestId("acquaintance-word-slide")).toBeVisible({ timeout: 30_000 });
            await expect(page.getByTestId("lesson-word-example")).toBeVisible({ timeout: 30_000 });
            await page.waitForTimeout(1_500);
            await sink.shot("acquaintance-example");
            acquaintanceShot = true;

            // Walk the hand to the REVIEW body of the lesson and capture the
            // compact example on an answer side too.
            const { completeTrainingUntilCriterion, completeAcquaintanceHandIfPresent } =
                await import("../../helpers/lesson.ts");
            const nextBtn = page.getByTestId("acquaintance-next-btn");
            for (let index = 0; index < 24; index++) {
                const trainingVisible = await page
                    .getByTestId("acquaintance-training")
                    .isVisible({ timeout: 800 })
                    .catch(() => false);
                if (trainingVisible) break;
                await nextBtn.click({ timeout: 2_000 }).catch(() => undefined);
            }
            await completeTrainingUntilCriterion(page);
            await completeAcquaintanceHandIfPresent(page);

            for (let index = 0; index < 12 && !answerShot; index++) {
                try {
                    await lesson.showAnswer();
                    await expect(page.getByTestId("lesson-word-example")).toBeVisible({ timeout: 8_000 });
                    await page.waitForTimeout(800);
                    await sink.shot("answer-example");
                    answerShot = true;
                } catch {
                    await lesson.clickNextCard().catch(() => undefined);
                }
            }
        } catch {
            // The lesson schedule may open with a different view — walk the
            // answers instead.
        }

        for (let index = 0; index < 8 && !answerShot; index++) {
            try {
                await lesson.showAnswer();
                await expect(page.getByTestId("lesson-word-example")).toBeVisible({ timeout: 8_000 });
                await page.waitForTimeout(800);
                await sink.shot("answer-example");
                answerShot = true;
            } catch {
                await lesson.clickNextCard().catch(() => undefined);
            }
        }
        expect(answerShot || acquaintanceShot, "at least one lesson example screenshot").toBe(true);
    },
};
