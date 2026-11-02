import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { captureReviewBody } from "./review-body.ts";

/**
 * The review body of a lesson on a due profile.
 *
 * The `due-corpus` preset makes kanji and example cards due today, so
 * the lesson core is a random mix of reading/meaning quizzes, yes/no
 * cards and normal showings — the randomized part of the lesson the
 * scripted asserts cannot pin down and the semantic examiner grades
 * (the `lesson-review-body:review-body-*` wildcard in the exam model).
 *
 * The writing view (stroke practice) is part of that mix and cannot be
 * advanced past — its reveal IS the drawing, there is no skip control.
 * When it holds the first card the walk collects nothing, so the
 * scenario re-enters the lesson — a reload regenerates every view for
 * free, the seeded profile survives in localStorage (the same reroll
 * lesson-reading-quiz walks). The budget is honest: if no frames land
 * within it the error says exactly that, it does not blame selectors.
 */
const REROLL_BUDGET = 5;
const HAND_WAIT_FIRST_MS = 45_000;
const HAND_WAIT_RETRY_MS = 10_000;

export const lessonReviewBodyScenario: ShotScenario = {
    ...descriptors["lesson-review-body"],
    async run({ page }, sink) {
        for (let attempt = 0; attempt < REROLL_BUDGET; attempt++) {
            await page.goto("/lesson");
            await page.getByTestId("lesson-page").waitFor({ state: "visible", timeout: 90_000 });

            // A due profile may open straight into the review body, without
            // an acquaintance hand — wait briefly for the hand, then proceed
            // either way. After the first attempt the profile has proven
            // itself, so retries wait shorter.
            await page
                .getByTestId("acquaintance-view")
                .waitFor({
                    state: "visible",
                    timeout: attempt === 0 ? HAND_WAIT_FIRST_MS : HAND_WAIT_RETRY_MS,
                })
                .catch(() => undefined);

            const captured = await captureReviewBody(page, sink, 6);
            if (captured > 0) return;
            console.log(`[shots] attempt ${attempt}: no review-body frames captured, rerolling`);
        }
        throw new Error(`no review-body frames captured in ${REROLL_BUDGET} attempts`);
    },
};
