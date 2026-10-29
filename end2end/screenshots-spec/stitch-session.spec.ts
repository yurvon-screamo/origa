/**
 * #749 final screenshots: the RU insight ledger of the home screen, whole
 * page, with a seeded demo corpus.
 *
 * The screen itself lives in `end2end/shots/scenarios/stitch-session.ts` —
 * this file is the Playwright entry point for it.
 */
import { test } from "@playwright/test";

import { runScenarioWithManifest } from "../shots/runner.ts";

test("capture home insights with seeded verdicts", async ({ browser }) => {
    test.setTimeout(1_200_000);
    await runScenarioWithManifest(browser, "stitch-session", {
        outDir: process.env.SHOT_OUT_DIR ?? "/tmp/opencode/shots/stitch-session",
        label: "stitch-session",
        cdnBaseUrl: process.env.SHOT_CDN_BASE_URL ?? "",
        commit: process.env.SHOT_COMMIT ?? "unknown",
    });
});
