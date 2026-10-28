import { expect, type Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The kanji slide of the acquaintance hand.
 *
 * The seeded hand of the acquaintance-hand preset holds one kanji card, so
 * the presentation walk reaches it deterministically. The capture expands
 * the «Подробнее» section before the shot: the collapsed slide shows the
 * reading rows (ON/KUN labels with the ru transliteration) and the meaning
 * row — the label columns the overflow fix (task I7_9cVuw6e) targets —
 * while the expanded section exposes the radicals and writing rows that
 * share the same label-column recipe.
 */
export const lessonAcquaintanceKanjiScenario: ShotScenario = {
    ...descriptors["lesson-acquaintance-kanji"],
    async run({ page }, sink) {
        await page.goto("/lesson");
        await page.getByTestId("lesson-page").waitFor({ state: "visible", timeout: 90_000 });

        // The hand appears after the lesson data prep — allow a long window.
        const hand = page.getByTestId("acquaintance-view");
        await hand.waitFor({ state: "visible", timeout: 180_000 });

        // Presentation → «Дальше» until the kanji slide shows. The whole
        // hand is presented card by card, so the kanji slide is guaranteed
        // to appear before the training takes over.
        const next = page.getByTestId("acquaintance-next-btn");
        let kanjiShot = false;
        for (let index = 0; index < 12 && !kanjiShot; index++) {
            if (
                await page
                    .getByTestId("acquaintance-kanji-slide")
                    .isVisible()
                    .catch(() => false)
            ) {
                // Let the furigana pass settle before the capture.
                await page.waitForTimeout(700);
                const toggle = page.getByTestId("kanji-details-toggle");
                if (await toggle.isVisible().catch(() => false)) {
                    await toggle.click();
                    // The expanded section (radicals/writing/examples) renders
                    // below the fold — give the layout a beat before the shot.
                    await page.waitForTimeout(400);
                }
                await sink.shot("kanji-presentation", { fullPage: true });
                kanjiShot = true;
                break;
            }
            await next.click({ timeout: 2_000 }).catch(() => undefined);
            await page.waitForTimeout(500);
        }
        if (!kanjiShot) {
            console.log(
                "[shots] no kanji slide captured, visible test ids:",
                JSON.stringify(await visibleTestIds(page)),
            );
        }
        expect(kanjiShot, "the hand must contain a kanji slide").toBe(true);
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
