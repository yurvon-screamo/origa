import { type Locator, type Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** Clicks the first of the given test ids that is on screen. */
async function clickFirstVisible(page: Page, testIds: string[], timeout = 1_500): Promise<string | null> {
    for (const id of testIds) {
        const locator = page.getByTestId(id);
        // isVisible() returns immediately (ignores timeouts) — a mounting
        // card would read as absent. waitFor actually waits.
        const visible = await locator
            .waitFor({ state: "visible", timeout })
            .then(() => true)
            .catch(() => false);
        if (visible) {
            await locator.click({ timeout: 3_000 });
            return id;
        }
    }
    return null;
}

/** The question kinds of the review body, in the order the walk checks them. */
const QUESTION_MARKERS: Array<[string, string]> = [
    ["quiz", 'button[data-testid^="quiz-option-"]'],
    ["yesno", '[data-testid="yesno-yes-btn"]'],
    ["audio-recall", '[data-testid="audio-recall-play-btn"]'],
    ["counter-bindings", '[data-testid="counter-binding-front"]'],
    ["normal", '[data-testid="lesson-show-answer-btn"]'],
];

const REVEALS = [
    "lesson-show-answer-btn",
    "quiz-option-0",
    "yesno-no-btn",
    "audio-recall-reveal-btn",
    "lesson-example-reveal",
    "counter-binding-show-answer-btn",
];

const ADVANCES = [
    "lesson-card-next-btn",
    "quiz-submit-btn",
    "lesson-rating-btn-again",
    "audio-recall-know-btn",
    "lesson-example-understood",
    "lesson-example-next",
];

async function firstVisible(page: Page, selector: string, timeout = 2_000): Promise<Locator | null> {
    const locator = page.locator(selector).first();
    const visible = await locator
        .waitFor({ state: "visible", timeout })
        .then(() => true)
        .catch(() => false);
    return visible ? locator : null;
}

/**
 * The unified lesson surfaces: the acquaintance presentation, the
 * training rounds with the revealed answer, the completion screen and the
 * review body cards.
 */
export const lessonUnificationScenario: ShotScenario = {
    ...descriptors["lesson-unification"],
    async run({ page }, sink) {
        // The hand appears after the lesson data prep — allow a long window.
        const hand = page.getByTestId("acquaintance-view");
        const handVisible = await hand
            .waitFor({ state: "visible", timeout: 90_000 })
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

        // The review body: every card is revealed, captured and advanced.
        for (let card = 1; card <= 8; card++) {
            const completeShown = await page
                .getByTestId("lesson-complete-screen")
                .waitFor({ state: "visible", timeout: 2_000 })
                .then(() => true)
                .catch(() => false);
            if (completeShown) break;

            let kind = "unknown";
            for (const [name, selector] of QUESTION_MARKERS) {
                if (await firstVisible(page, selector)) {
                    kind = name;
                    break;
                }
            }
            await clickFirstVisible(page, REVEALS, 3_000);
            await page.waitForTimeout(700);
            await sink.shot(`review-body-${kind}-${card}`);
            await clickFirstVisible(page, ADVANCES, 3_000);
        }
    },
};
