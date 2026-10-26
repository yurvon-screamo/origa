import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The counter bindings pack (issue #415): the common-counter slot that
 * walks «number × suffix» showings.
 *
 * The `counter-due` preset seeds a single due 本 counter card, so the
 * lesson opens straight on the pack: the question side («2×本»), the
 * reading answer (the registry reading as the main text, the rating
 * pair below the card) and the expanded readings table.
 */
export const lessonCounterPackScenario: ShotScenario = {
    ...descriptors["lesson-counter-pack"],
    async run({ page }, sink) {
        await page.goto("/lesson");
        await page.getByTestId("lesson-page").waitFor({ state: "visible", timeout: 90_000 });

        const pack = page.getByTestId("counter-binding-front");
        await pack.waitFor({ state: "visible", timeout: 90_000 });
        await page.waitForTimeout(700);
        await sink.shot("pack-front");

        // Reveal the reading answer (the shared Space pipeline).
        await page.keyboard.press(" ");
        await page.getByTestId("counter-binding-answer").waitFor({ timeout: 10_000 });
        await page.waitForTimeout(700);
        await sink.shot("pack-answer");

        // The full readings table behind the expand toggle (the answered
        // row keeps its accent).
        await page.getByTestId("counter-binding-mutations-toggle").click();
        await page.getByTestId("counter-binding-mutations-table").waitFor({ timeout: 10_000 });
        await page.waitForTimeout(700);
        await sink.shot("pack-answer-table");
    },
};
