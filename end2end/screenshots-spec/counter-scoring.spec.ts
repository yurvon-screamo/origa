/**
 * Screenshots for #808: the counter-suffix card in the onboarding scoring.
 *
 * The screen itself lives in `end2end/shots/scenarios/counter-scoring.ts`
 * — this file is the Playwright entry point for it. The scenario walks the
 * N5 scoring queue to the counter card; `SHOT_LABEL=before` captures the
 * earlier build (served on another port through SHOT_BASE_URL).
 *
 * Run: bunx playwright test screenshots-spec/counter-scoring.spec.ts -c playwright.screenshots.config.ts
 * Env: SHOT_LABEL=after|before, SHOT_OUT_DIR=/tmp/opencode/screens-808
 */
import { test } from "@playwright/test";

import { runScenarioWithManifest } from "../shots/runner.ts";

test("counter scoring card screenshots", async ({ browser }) => {
    test.setTimeout(1_200_000);
    await runScenarioWithManifest(browser, "counter-scoring", {
        outDir: process.env.SHOT_OUT_DIR ?? "/tmp/opencode/shots/counter-scoring",
        label: "counter-scoring",
        cdnBaseUrl: process.env.SHOT_CDN_BASE_URL ?? "",
        commit: process.env.SHOT_COMMIT ?? "unknown",
        baseURL: process.env.SHOT_BASE_URL || undefined,
    });
});
