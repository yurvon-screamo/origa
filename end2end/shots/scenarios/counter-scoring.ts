import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

const COUNTER_TAG = "Счётный суффикс";

/**
 * The counter card in the interactive assessment.
 *
 * The counters are the LAST section of the queue (sort order 4), so the
 * walk dismisses every preceding card with «Не знаю» — the per-click skip
 * is persisted, so the walk is resumable in principle.
 */
export const counterScoringScenario: ShotScenario = {
    ...descriptors["counter-scoring"],
    async run({ page }, sink) {
        // The screen name must not depend on the run label: the exam model
        // and the registry address screens by `scenario:screen`, and
        // `card-${SHOT_LABEL}` made the key different on every run.
        const variant = process.env.SHOT_COUNTER_VARIANT ?? "after";
        const isBefore = variant === "before";

        const step = page.getByTestId("onboarding-scoring-step");
        const tag = step.locator("span.tag").first();
        const dontKnow = page.getByTestId("scoring-step-dont-know");
        let reachedCounter = false;
        for (let index = 0; index < 2500; index++) {
            const cardTag = ((await tag.textContent().catch(() => null)) ?? "").trim();
            if (!cardTag) break; // queue exhausted — the counter was not reached
            if (cardTag === COUNTER_TAG) {
                reachedCounter = true;
                break;
            }
            await dontKnow.click();
            await expect(dontKnow).toBeEnabled({ timeout: 10_000 });
        }
        expect(reachedCounter, "counter card must appear in the N5 scoring queue").toBe(true);

        // The fix's DOM contract: the counter question is plain text (no
        // dictionary furigana) and the card offers no audio button. On the
        // earlier build the inverse holds.
        if (isBefore) {
            await expect(page.getByTestId("scoring-step-question")).toBeVisible();
            await expect(page.getByTestId("scoring-step-audio")).toBeVisible();
        } else {
            await expect(page.getByTestId("scoring-step-question-plain")).toBeVisible();
            await expect(page.getByTestId("scoring-step-audio")).toHaveCount(0);
        }

        await page.waitForTimeout(500);
        await sink.shot(`card-${variant}`);
    },
};
