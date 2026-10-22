/**
 * Lesson screenshots for #528: the acquaintance slide and the answer side
 * of the lesson body with the word example.
 *
 * The screen itself lives in `end2end/shots/scenarios/lesson-example.ts` —
 * this file is the Playwright entry point for it.
 */
import { test } from "@playwright/test";

import { runScenarioWithManifest } from "../shots/runner.ts";

test("lesson example screenshots", async ({ browser }) => {
    test.setTimeout(600_000);
    await runScenarioWithManifest(browser, "lesson-example", {
        outDir: process.env.SHOT_OUT_DIR ?? "/tmp/opencode/shots/lesson-example",
        label: "lesson-example",
        cdnBaseUrl: process.env.SHOT_CDN_BASE_URL ?? "",
        commit: process.env.SHOT_COMMIT ?? "unknown",
    });
});
