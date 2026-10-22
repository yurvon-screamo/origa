/**
 * Screenshots for issue #809: the due-card counter in the header of the
 * "Обзор дня" widget on the home page.
 *
 * The screen itself lives in `end2end/shots/scenarios/home-due-counter.ts`
 * — this file is the Playwright entry point for it. The seed makes the
 * number deterministic: the onboarding "mark all known" gives every
 * corpus card a review date of YESTERDAY, so the scenario rewrites the
 * dates — every card gets a future date, and exactly 37 cards get a past
 * one.
 */
import { test } from "@playwright/test";

import { runScenarioWithManifest } from "../shots/runner.ts";

test("home due counter screenshots with seeded data", async ({ browser }) => {
    test.setTimeout(900_000);
    await runScenarioWithManifest(browser, "home-due-counter", {
        outDir: process.env.SHOT_OUT_DIR ?? "/tmp/opencode/shots/home-due-counter",
        label: "home-due-counter",
        cdnBaseUrl: process.env.SHOT_CDN_BASE_URL ?? "",
        commit: process.env.SHOT_COMMIT ?? "unknown",
    });
});
