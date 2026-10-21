import { readFile } from "node:fs/promises";
import { join } from "node:path";
import type { Page } from "@playwright/test";

import { readPngSize, sha256Hex, shotFileName, type ShotEntry } from "./manifest.ts";
import type { ShotSink } from "./types.ts";
import type { ViewportTag } from "./viewports.ts";

export interface SinkOptions {
    outDir: string;
    scenario: string;
    title: string;
    route: string;
    viewport: ViewportTag;
    locale: string;
    baseURL: string;
    cdnBaseUrl: string;
    commit: string;
}

/**
 * The sink a scenario writes its screens into.
 *
 * It owns the file, the hash and the provenance: a scenario only says
 * which screen it wants, never where the file goes.
 */
export function createSink(page: Page, options: SinkOptions): ShotSink {
    const entries: ShotEntry[] = [];
    return {
        entries,
        shot: async (name: string, shotOptions: { fullPage?: boolean } = {}) => {
            const fullPage = shotOptions.fullPage ?? false;
            const file = join(options.outDir, shotFileName(options.scenario, name, options.viewport));
            await page.screenshot({ path: file, fullPage });
            const data = await readFile(file);
            const size = readPngSize(data);
            const entry: ShotEntry = {
                scenario: options.scenario,
                title: options.title,
                screen: name,
                route: options.route,
                viewport: options.viewport,
                locale: options.locale,
                width: size.width,
                height: size.height,
                fullPage,
                bytes: data.byteLength,
                sha256: sha256Hex(data),
                file,
                capturedAt: new Date().toISOString(),
                commit: options.commit,
                baseURL: options.baseURL,
                cdnBaseUrl: options.cdnBaseUrl,
            };
            entries.push(entry);
            return entry;
        },
    };
}
