/**
 * Screenshots for issue #528 (word examples): the word detail page of a
 * translated word — hero, translations and examples.
 *
 * The screen itself lives in `end2end/shots/scenarios/word-examples.ts` —
 * this file is the Playwright entry point for it. SHOT_WORD picks the
 * word (あさ by default).
 */
import { test } from "@playwright/test";

import { runScenarioWithManifest } from "../shots/runner.ts";

test("word detail screenshots", async ({ browser }) => {
    test.setTimeout(600_000);
    await runScenarioWithManifest(browser, "word-examples", {
        outDir: process.env.SHOT_OUT_DIR ?? "/tmp/opencode/shots/word-examples",
        label: "word-examples",
        cdnBaseUrl: process.env.SHOT_CDN_BASE_URL ?? "",
        commit: process.env.SHOT_COMMIT ?? "unknown",
    });
});
