/**
 * Screenshots for fl2O037UR7 (home accuracy chart): a test user completes
 * the onboarding (mark all known → the corpus is in the set), the typical
 * active-user state is seeded into the IndexedDB record, and the home
 * screen is captured with a live 30-day accuracy line.
 *
 * The screen itself lives in `end2end/shots/scenarios/home-accuracy-chart.ts` —
 * this file is the Playwright entry point for it.
 */
import { test } from "@playwright/test";

import { runScenarioWithManifest } from "../shots/runner.ts";

test("home accuracy chart screenshots with seeded data", async ({ browser }) => {
    test.setTimeout(900_000);
    await runScenarioWithManifest(browser, "home-accuracy-chart", {
        outDir: process.env.SHOT_OUT_DIR ?? "/tmp/opencode/shots/home-accuracy-chart",
        label: "home-accuracy-chart",
        cdnBaseUrl: process.env.SHOT_CDN_BASE_URL ?? "",
        commit: process.env.SHOT_COMMIT ?? "unknown",
    });
});
