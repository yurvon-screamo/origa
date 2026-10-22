/**
 * Screenshots for issue #749 (home insight panels): a test user completes
 * the onboarding (mark all known → the corpus is in the set), the typical
 * active-user state is seeded into the IndexedDB record, and the home
 * screen is captured with live verdicts.
 *
 * The screen itself lives in `end2end/shots/scenarios/home-insights.ts` —
 * this file is the Playwright entry point for it.
 */
import { test } from "@playwright/test";

import { runScenarioWithManifest } from "../shots/runner.ts";

test("home insights screenshots with seeded data", async ({ browser }) => {
    test.setTimeout(900_000);
    await runScenarioWithManifest(browser, "home-insights", {
        outDir: process.env.SHOT_OUT_DIR ?? "/tmp/opencode/shots/home-insights",
        label: "home-insights",
        cdnBaseUrl: process.env.SHOT_CDN_BASE_URL ?? "",
        commit: process.env.SHOT_COMMIT ?? "unknown",
    });
});
