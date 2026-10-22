import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The example recall card (#528): the sentence question, the reveal and
 * the token-translator answer with the translation.
 *
 * The `due-examples` preset injects due example SRS cards and pushes
 * everything else out, so the lesson consists of example recall cards
 * only — the first card is the screen.
 *
 * Issue #822 re-centered this card: a flex-grow leftover pinned the
 * sentence to the top edge and the reveal button to the bottom one; the
 * capture shows the centered question/answer group.
 */
export const lessonExampleRecallScenario: ShotScenario = {
    ...descriptors["lesson-example-recall"],
    async run({ page }, sink) {
        await page.goto("/lesson");
        await page.getByTestId("lesson-page").waitFor({ state: "visible", timeout: 90_000 });

        // Three example cards are seeded from ONE chunk (s0000). The
        // chunk loader has a known inflight race: a second consumer of a
        // chunk gets an instant error while the first fetch is still in
        // flight, so the FIRST card may render the «unavailable» notice.
        // The chunk stays cached for the session — the next card renders
        // deterministically. Advance past any unavailable card.
        for (let card = 0; card < 3; card++) {
            const unavailable = await page
                .getByTestId("lesson-example-unavailable")
                .isVisible()
                .catch(() => false);
            if (unavailable) {
                console.log(`[shots] card ${card}: chunk race hit, advancing`);
                await page.getByTestId("lesson-example-next").click({ timeout: 8_000 });
                await page.waitForTimeout(700);
                continue;
            }
            const sentence = await page
                .getByTestId("lesson-example-ja")
                .waitFor({ state: "visible", timeout: 45_000 })
                .then(() => true)
                .catch(() => false);
            if (!sentence) continue;

            await page.waitForTimeout(900);
            await sink.shot("example-question");

            await page
                .getByTestId("lesson-example-reveal")
                .click({ timeout: 10_000 });
            // The token translator resolves asynchronously.
            await page.waitForTimeout(2_000);
            await sink.shot("example-answer");

            const questionShown = await page
                .getByTestId("lesson-example-sentence")
                .isVisible()
                .catch(() => false);
            expect(questionShown, "the token-translated sentence must render").toBe(true);
            return;
        }

        expect(
            await page.getByTestId("lesson-example-recall").isVisible().catch(() => false),
            "an example card must render within the seeded hand",
        ).toBe(true);
    },
};
