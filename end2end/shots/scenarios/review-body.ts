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
    // Last: a known interactive kind must win when selectors overlap, so
    // every non-writing card pays no extra probe — a writing card alone
    // waits out the five misses above (~10 s) before its own hit. Class
    // selector: the drawing practice mounts without a test_id
    // (KanjiDrawingPractice defaults to an empty test_id and
    // WritingCard passes none), so the canvas carries no testid — the
    // same marker lesson-reading-quiz uses.
    ["writing", "canvas.kanji-drawing-canvas"],
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

/** The question kind of the current card, "unknown" when no marker matches. */
async function detectKind(page: Page): Promise<string> {
    for (const [name, selector] of QUESTION_MARKERS) {
        if (await firstVisible(page, selector)) {
            return name;
        }
    }
    return "unknown";
}

/**
 * Walks the review body: every card is revealed, captured and advanced.
 *
 * The captures are named `review-body-<kind>-<card>` — the kind depends
 * on the scheduler's random pick, which is exactly the coverage the
 * semantic examiner grades (see the `review-body-*` wildcard in the
 * exam model).
 *
 * Returns the number of captured frames: 0 means the walk collected
 * nothing (a writing card held the first screen, or the lesson was
 * already complete) — the caller decides between a reroll and an
 * honest exit.
 */
export async function captureReviewBody(page: Page, sink: ShotSink, maxCards = 8): Promise<number> {
    let captured = 0;
    for (let card = 1; card <= maxCards; card++) {
        const completeShown = await page
            .getByTestId("lesson-complete-screen")
            .waitFor({ state: "visible", timeout: 2_000 })
            .then(() => true)
            .catch(() => false);
        if (completeShown) break;

        const kind = await detectKind(page);
        // The writing card reveals by DRAWING strokes — it has no reveal
        // control at all, so the reveal probe below can never pass and a
        // walk cannot advance past it (lesson-reading-quiz rerolls for
        // the same reason). Stop here BEFORE the probe: with the check
        // buried in the no-reveal branch a first-screen writing card
        // would burn ~18 s of pointless probing and then die on the
        // stale-selector guard. No frame either: the exam model grades
        // the review-body wildcard against reveal/rating controls a
        // canvas screen does not show — an uncalibrated capture would
        // read as a finding.
        if (kind === "writing") {
            console.log(`[shots] card ${card}: writing card, the walk stops here`);
            break;
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
        captured += 1;
        const advanced = await clickFirstVisible(page, ADVANCES, 3_000).catch(() => null);
        if (!advanced) break;
    }
    return captured;
}
