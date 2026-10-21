import { expect, type Page } from "@playwright/test";

import { LessonPage } from "../../pages/lesson.page.ts";
import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The lesson in its two shapes: the acquaintance slide of the hand and the
 * answer side of the review body.
 *
 * The profile behind the screens has exactly one word (see the
 * `with-word` preset), so the hand is a single slide.
 */
export const lessonExampleScenario: ShotScenario = {
    ...descriptors["lesson-example"],
    async run({ page }, sink) {
        const lesson = new LessonPage(page);
        // The lesson route builds its own hand. (The old spec navigated
        // through a link named "Words" — the interface is in Russian, so
        // that wait never resolved: the spec had rotted unnoticed.)
        await page.goto("/lesson");
        await page.getByTestId("lesson-page").waitFor({ state: "visible", timeout: 90_000 });
        await lesson.expectLessonVisible();

        let acquaintanceShot = false;
        let answerShot = false;

        // The hand shows several slides (kanji, word, grammar, counter).
        // Walk it to the word slide — the screen the example-sentence
        // feature (issue #528) landed on. The hand must be on screen
        // first: the lesson opens with a loading state, and a wait for a
        // marker during that load reads as absent.
        const hand = page.getByTestId("acquaintance-view");
        await hand.waitFor({ state: "visible", timeout: 90_000 });
        for (let index = 0; index < 12 && !acquaintanceShot; index++) {
            const wordSlide = page.getByTestId("acquaintance-word-slide");
            const visible = await wordSlide
                .waitFor({ state: "visible", timeout: index === 0 ? 30_000 : 8_000 })
                .then(() => true)
                .catch(() => false);
            if (visible) {
                // The example line of the slide loads from the CDN
                // asynchronously. When the examples index is not
                // available, the slide renders without it — the screen is
                // still the screen.
                await page
                    .getByTestId("lesson-word-example")
                    .waitFor({ state: "visible", timeout: 15_000 })
                    .catch(() => undefined);
                await page.waitForTimeout(1_500);
                await sink.shot("acquaintance-example");
                acquaintanceShot = true;
                break;
            }
            const next = page.getByTestId("acquaintance-next-btn");
            if (!(await next.isVisible().catch(() => false))) break;
            await next.click({ timeout: 3_000 }).catch(() => undefined);
            await page.waitForTimeout(700);
        }

        // Walk the hand to the REVIEW body of the lesson and capture the
        // answer side.
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
                await expect(page.getByTestId("lesson-card")).toBeVisible({ timeout: 8_000 });
                await page.waitForTimeout(800);
                await sink.shot("answer-example");
                answerShot = true;
            } catch {
                await lesson.clickNextCard().catch(() => undefined);
            }
        }

        if (!answerShot && !acquaintanceShot) {
            // The lesson opens with a different view than the expectation
            // (the schedule of a fresh profile changed): report what IS on
            // screen, so the next reader of the log does not have to guess.
            console.log(
                "[shots] the lesson markers are not on screen, visible test ids:",
                JSON.stringify(await visibleTestIds(page)),
            );
        }
        expect(answerShot || acquaintanceShot, "at least one lesson example screenshot").toBe(true);
    },
};

/** The test ids the lesson page currently renders. */
async function visibleTestIds(page: Page): Promise<string[]> {
    return await page.evaluate(() =>
        Array.from(document.querySelectorAll("[data-testid]"))
            .map((element) => element.getAttribute("data-testid") ?? "")
            .filter(Boolean)
            .slice(0, 60),
    );
}
