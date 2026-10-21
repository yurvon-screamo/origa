import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { countDue, readStoredUser, seedDueCards, writeStoredUser } from "../seed.ts";

/** The number of due cards the seeded state produces. */
const DUE_TARGET = 37;

/**
 * The due counter in the header of the day overview.
 *
 * Two states are captured: the day-one state a fresh profile really lands
 * in (the whole imported corpus is past its review date) and a seeded
 * state with an exact number of due cards.
 */
export const homeDueCounterScenario: ShotScenario = {
    ...descriptors["home-due-counter"],
    async run({ page }, sink) {
        const stored = await readStoredUser(page);
        const dayOne = countDue(JSON.parse(stored.value), new Date());

        const counter = page.getByTestId("home-today-overview-due-count");
        await expect(counter).toBeVisible({ timeout: 120_000 });
        // Exact number: a plain substring check would also pass for "137".
        await expect(counter).toHaveText(new RegExp(`\\b${dayOne}\\b`), { timeout: 120_000 });
        await page.waitForTimeout(1_500);
        await sink.shot("due-day-one");

        const seeded = seedDueCards(stored.value, new Date(), DUE_TARGET);
        expect(seeded.due, "the seed must produce exactly DUE_TARGET due cards").toBe(DUE_TARGET);
        await writeStoredUser(page, stored.key, seeded.json);

        // The reload re-reads the local record. The mount sync merges the
        // server snapshot with max()/LWW semantics — the seeded future
        // dates survive because every patched card carries a bumped
        // `last_review_date`.
        await page.reload({ waitUntil: "domcontentloaded" });
        await expect(counter).toBeVisible({ timeout: 120_000 });
        await expect(counter).toHaveText(new RegExp(`\\b${DUE_TARGET}\\b`), { timeout: 120_000 });
        await page.waitForTimeout(1_500);
        await sink.shot("due-counter");
    },
};
