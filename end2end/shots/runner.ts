import type { Browser } from "@playwright/test";
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

import { detectCdnBaseUrl } from "./bundle.ts";
import { createSink } from "./capture.ts";
import type { ShotEntry, ShotManifest } from "./manifest.ts";
import { openShotSession } from "./session.ts";
import { getImplementation } from "./scenarios/index.ts";
import { DEFAULT_VIEWPORTS, type ViewportTag } from "./viewports.ts";

export interface RunOptions {
    outDir: string;
    /** Label of the run — the file name of the manifest. */
    label?: string;
    /** CDN URL the bundle is expected to point at. */
    cdnBaseUrl: string;
    commit: string;
    viewports?: ViewportTag[];
    locale?: string;
    baseURL?: string;
}

/**
 * Captures one scenario in every requested viewport.
 *
 * Each viewport gets its own browser context: the app caches the loaded
 * dictionaries in IndexedDB, and a reused context would show the second
 * viewport a warmed-up app instead of the one a phone user sees.
 */
export async function runScenario(
    browser: Browser,
    id: string,
    options: RunOptions,
): Promise<ShotEntry[]> {
    const scenario = getImplementation(id);
    const viewports = options.viewports ?? scenario.viewports ?? DEFAULT_VIEWPORTS;
    const locale = options.locale ?? scenario.locale;
    const baseURL =
        options.baseURL ?? scenario.baseURL ?? process.env.E2E_APP_BASE_URL ?? "http://localhost:1420";
    const cdnBaseUrl = await detectCdnBaseUrl(baseURL, options.cdnBaseUrl);
    const entries: ShotEntry[] = [];

    for (const viewport of viewports) {
        const session = await openShotSession({ browser, baseURL, locale, viewport, preset: scenario.preset });
        try {
            const sink = createSink(session.page, {
                outDir: options.outDir,
                scenario: scenario.id,
                title: scenario.title,
                route: scenario.route,
                viewport,
                locale,
                baseURL,
                cdnBaseUrl,
                commit: options.commit,
            });
            await scenario.run({ page: session.page, context: session.context, baseURL, locale, viewport }, sink);
            entries.push(...sink.entries);
        } finally {
            await session.close();
        }
    }

    return entries;
}

/**
 * Captures a scenario and writes its manifest.
 *
 * The manifest is the hand-off between the browser process and the CLI:
 * the files live in the output directory, the manifest tells what they
 * are.
 */
export async function runScenarioWithManifest(
    browser: Browser,
    id: string,
    options: RunOptions,
): Promise<ShotEntry[]> {
    const entries = await runScenario(browser, id, options);
    const manifest: ShotManifest = { version: 1, label: options.label ?? id, entries };
    await mkdir(options.outDir, { recursive: true });
    await writeFile(
        join(options.outDir, `manifest.${id}.json`),
        `${JSON.stringify(manifest, null, 2)}\n`,
    );
    return entries;
}
