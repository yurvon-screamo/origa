import { type Locator, type Page } from "@playwright/test";

import type { ShotSink } from "../types.ts";

/** Clicks the first of the given test ids that is on screen. */
export async function clickFirstVisible(page: Page, testIds: string[], timeout = 1_500): Promise<string | null> {
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
export const QUESTION_MARKERS: Array<[string, string]> = [
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
 * Walks the review body: every card is revealed, captured and advanced.
 *
 * The captures are named `review-body-<kind>-<card>` — the kind depends
 * on the scheduler's random pick, which is exactly the coverage the
 * semantic examiner grades (see the `review-body-*` wildcard in the
 * exam model).
 */
export async function captureReviewBody(page: Page, sink: ShotSink, maxCards = 8): Promise<void> {
    for (let card = 1; card <= maxCards; card++) {
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
        // The review body is scheduler-randomized: a card kind whose
        // controls this walk does not know must stop the walk, not fail
        // the capture — the exam grades what was collected. A walk that
        // cannot reveal even its FIRST card collected nothing at all —
        // that is a broken selector, and it must fail loudly instead of
        // silently shrinking the coverage.
        const revealed = await clickFirstVisible(page, REVEALS, 3_000).catch(() => null);
        if (!revealed) {
            if (card === 1) {
                throw new Error("the review body did not reveal its first card — the walk selectors are stale");
            }
            break;
        }
        await page.waitForTimeout(700);
        await sink.shot(`review-body-${kind}-${card}`);
        const advanced = await clickFirstVisible(page, ADVANCES, 3_000).catch(() => null);
        if (!advanced) break;
    }
}
