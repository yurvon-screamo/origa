import { expect, type Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The answer side of a grammar slide inside the acquaintance hand.
 *
 * The seeded hand of the acquaintance-hand preset holds word, kanji, grammar and
 * counter slides; the training rotates them until each meets its
 * criterion. The walk reveals every answer and keeps the one with the
 * grammar details toggle — the «Подробнее» trigger that issue #822
 * centered (it used to be pinned left by a text-left wrapper).
 *
 * 8KdX3bW3dt: the grammar cards are voiced now (header audio button in
 * the presentation and in the training), so the walk also keeps the
 * grammar presentation slide with its button.
 */
export const lessonAcquaintanceGrammarScenario: ShotScenario = {
    ...descriptors["lesson-acquaintance-grammar"],
    async run({ page }, sink) {
        await page.goto("/lesson");
        await page.getByTestId("lesson-page").waitFor({ state: "visible", timeout: 90_000 });

        // The hand appears after the lesson data prep — allow a long window.
        const hand = page.getByTestId("acquaintance-view");
        await hand.waitFor({ state: "visible", timeout: 180_000 });

        // Presentation → «Дальше» until the training takes over.
        const next = page.getByTestId("acquaintance-next-btn");
        let presentationShot = false;
        for (let index = 0; index < 12; index++) {
            const training = await page
                .getByTestId("acquaintance-training")
                .isVisible({ timeout: 1_000 })
                .catch(() => false);
            if (training) break;
            // The grammar slide carries the header audio button since
            // 8KdX3bW3dt (autoplay + replay of the pattern): capture it
            // when the hand shows it. Optional — the mandatory artifact
            // of this scenario is the grammar answer below.
            if (
                !presentationShot &&
                (await page
                    .getByTestId("acquaintance-grammar-slide")
                    .isVisible()
                    .catch(() => false))
            ) {
                await page.waitForTimeout(700);
                await sink.shot("grammar-presentation");
                presentationShot = true;
            }
            await next.click({ timeout: 2_000 }).catch(() => undefined);
            await page.waitForTimeout(500);
        }

        // Training rounds: reveal each answer; keep the grammar one,
        // rate «помню» to move the rotation forward. The whole training
        // body unmounts for a moment between the cards of the hand —
        // a bare «not visible» there is a transition, not the end; the
        // only real ends are the hand-closed screens. The scenario is
        // done as soon as the grammar answer is captured.
        const completed = page
            .getByTestId("acquaintance-completed")
            .or(page.getByTestId("lesson-complete-screen"));
        let grammarShot = false;
        let idleChecks = 0;
        for (let index = 0; index < 120 && !grammarShot; index++) {
            if (await completed.isVisible().catch(() => false)) {
                console.log(`[shots] round ${index}: the hand is closed`);
                break;
            }
            const training = await page
                .getByTestId("acquaintance-training")
                .isVisible()
                .catch(() => false);
            if (!training) {
                await page.waitForTimeout(800);
                if (++idleChecks > 15) {
                    console.log(`[shots] round ${index}: training did not come back`);
                    break;
                }
                continue;
            }
            idleChecks = 0;
            const reveal = page.getByTestId("acquaintance-reveal-btn");
            if (!(await reveal.isVisible().catch(() => false))) {
                await page.waitForTimeout(600);
                if (++idleChecks > 15) {
                    console.log(`[shots] round ${index}: the reveal did not come back`);
                    break;
                }
                continue;
            }
            await reveal.click({ timeout: 3_000 }).catch(() => undefined);
            const answer = page.getByTestId("acquaintance-training-answer");
            await answer.waitFor({ state: "visible", timeout: 4_000 }).catch(() => undefined);

            if (
                await page
                    .getByTestId("acquaintance-grammar-details")
                    .isVisible()
                    .catch(() => false)
            ) {
                await page.waitForTimeout(700);
                await sink.shot("grammar-answer");
                grammarShot = true;
                break;
            }

            const rating = page.getByTestId("acquaintance-rating-remember");
            if (!(await rating.isVisible().catch(() => false))) continue;
            await rating.click({ timeout: 2_000 }).catch(() => undefined);
            await answer.waitFor({ state: "hidden", timeout: 3_000 }).catch(() => undefined);
        }

        if (!grammarShot) {
            console.log(
                "[shots] no grammar slide captured, visible test ids:",
                JSON.stringify(await visibleTestIds(page)),
            );
        }
        expect(grammarShot, "the hand must contain a grammar slide").toBe(true);
    },
};

/** The test ids the page currently renders. */
async function visibleTestIds(page: Page): Promise<string[]> {
    return await page.evaluate(() =>
        Array.from(document.querySelectorAll("[data-testid]"))
            .map((element) => element.getAttribute("data-testid") ?? "")
            .filter(Boolean)
            .slice(0, 60),
    );
}
