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
 */
export const lessonReviewBodyScenario: ShotScenario = {
    ...descriptors["lesson-review-body"],
    async run({ page }, sink) {
        await page.goto("/lesson");
        await page.getByTestId("lesson-page").waitFor({ state: "visible", timeout: 90_000 });

        // A due profile may open straight into the review body, without
        // an acquaintance hand — wait briefly for the hand, then proceed
        // either way.
        await page
            .getByTestId("acquaintance-view")
            .waitFor({ state: "visible", timeout: 45_000 })
            .catch(() => undefined);

        await captureReviewBody(page, sink, 6);
    },
};
