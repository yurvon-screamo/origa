import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { captureReviewBody, clickFirstVisible } from "./review-body.ts";

/**
 * The unified lesson surfaces: the acquaintance presentation, the
 * training rounds with the revealed answer, the completion screen and the
 * review body cards.
 */
export const lessonUnificationScenario: ShotScenario = {
    ...descriptors["lesson-unification"],
    async run({ page }, sink) {
        // The lesson route builds its own hand.
        await page.goto("/lesson");
        await page.getByTestId("lesson-page").waitFor({ state: "visible", timeout: 90_000 });

        // The hand appears after the lesson data prep — allow a long window.
        // `acquaintance-view` wraps the whole hand (the first slide may be a
        // word, kanji, grammar or counter slide).
        const hand = page.getByTestId("acquaintance-view");
        const handVisible = await hand
            .waitFor({ state: "visible", timeout: 180_000 })
            .then(() => true)
            .catch(() => false);
        if (!handVisible) {
            throw new Error("the acquaintance hand must be on screen");
        }
        await page.waitForTimeout(700);
        await sink.shot("acquaintance-presentation");

        // Presentation → Next through the slides until the training takes over.
        for (let index = 0; index < 12; index++) {
            const training = await page
                .getByTestId("acquaintance-training")
                .isVisible({ timeout: 1_000 })
                .catch(() => false);
            if (training) break;
            const next = page.getByTestId("acquaintance-next-btn");
            if (!(await next.isVisible().catch(() => false))) break;
            await next.click({ timeout: 3_000 });
            await page.waitForTimeout(700);
        }

        // Training: reveal the answer and self-assess. Two rounds are
        // captured, then the hand is driven to completion.
        let rounds = 0;
        for (let index = 0; index < 60; index++) {
            const training = await page
                .getByTestId("acquaintance-training")
                .isVisible()
                .catch(() => false);
            if (!training) break;
            const revealed = await clickFirstVisible(page, ["acquaintance-reveal-btn"], 2_000);
            if (revealed) {
                rounds += 1;
                if (rounds <= 2) {
                    await page.waitForTimeout(700);
                    await sink.shot(`acquaintance-training-answer-${rounds}`);
                }
            }
            const rated = await clickFirstVisible(
                page,
                ["acquaintance-rating-remember", "acquaintance-reveal-btn", "acquaintance-next-btn"],
                2_000,
            );
            if (!rated) await page.waitForTimeout(500);
        }

        // The hand closes into the lesson complete screen (a fresh user has
        // no due review cards yet) or the review body — capture whichever
        // appears.
        const complete = page.getByTestId("lesson-complete-screen");
        const completeShown = await complete
            .waitFor({ state: "visible", timeout: 20_000 })
            .then(() => true)
            .catch(() => false);
        if (completeShown) {
            await page.waitForTimeout(700);
            await sink.shot("lesson-complete");
            return;
        }

        await captureReviewBody(page, sink);
    },
};
