/**
 * The screenshot driver.
 *
 * One generic spec captures every requested scenario — the scenarios
 * themselves live in `end2end/shots/`, so a new screen is a new entry in
 * the catalog, not a new spec file with its own copy of the login flow.
 *
 * Run through the CLI (`npm run shots -- capture --scenario <id>`), which
 * sets the environment below and starts this file directly.
 */
import { test } from "@playwright/test";

import { createExaminerFromEnv } from "../shots/examiner/examine.ts";
import { runScenarioWithManifest } from "../shots/runner.ts";
import type { ViewportTag } from "../shots/viewports.ts";

const requested = (process.env.SHOT_SCENARIOS ?? "")
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
const outDir = process.env.SHOT_OUT_DIR ?? "/tmp/opencode/shots";
const label = process.env.SHOT_LABEL ?? "shots";
const cdnBaseUrl = process.env.SHOT_CDN_BASE_URL ?? "";
const commit = process.env.SHOT_COMMIT ?? "unknown";
const locale = process.env.SHOT_LOCALE ?? "";
const baseUrl = process.env.SHOT_BASE_URL ?? "";
const viewports = (process.env.SHOT_VIEWPORTS ?? "")
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean) as ViewportTag[];

test.describe("screenshot scenarios", () => {
    if (requested.length === 0) {
        test.skip("SHOT_SCENARIOS is empty — nothing to capture", () => {});
        return;
    }

    for (const id of requested) {
        test(`shots: ${id}`, async ({ browser }) => {
            // A capture drives the whole app: login, onboarding, navigation.
            test.setTimeout(1_200_000);
            const examiner = await createExaminerFromEnv(process.env, outDir);
            await runScenarioWithManifest(browser, id, {
                outDir,
                label,
                cdnBaseUrl,
                commit,
                viewports: viewports.length > 0 ? viewports : undefined,
                locale: locale || undefined,
                baseURL: baseUrl || undefined,
                examiner: examiner ?? undefined,
            });
        });
    }
});
