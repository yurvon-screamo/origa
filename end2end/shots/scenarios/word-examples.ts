import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The word detail page of a translated word.
 *
 * The page shows the dictionary entry with its examples — the screen the
 * example-sentence feature (issue #528) landed on.
 */
export const wordExamplesScenario: ShotScenario = {
    ...descriptors["word-examples"],
    async run({ page }, sink) {
        await page.goto(`/words/${encodeURIComponent(process.env.SHOT_WORD ?? "あさ")}`);
        try {
            await expect(page.getByTestId("word-detail-word")).toBeVisible({ timeout: 60_000 });
        } catch {
            // A cold page load boots the WASM app from scratch; while the
            // session migration is in flight the protected route shows the
            // login screen even for an authenticated user. The bootstrap
            // persists the user locally, so one reload lands on the page.
            await page.waitForTimeout(25_000);
            await page.reload({ waitUntil: "domcontentloaded" });
            await expect(page.getByTestId("word-detail-word")).toBeVisible({ timeout: 90_000 });
        }
        await page.waitForTimeout(3_000); // examples chunk fetch
        await sink.shot("detail-full", { fullPage: true });
        await sink.shot("detail-viewport");
    },
};
