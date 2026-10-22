import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

import { shotKey, type ShotEntry } from "./manifest.ts";

/**
 * One capture in the registry.
 *
 * The current snapshot and the last successful upload live side by side:
 * a local capture that was never published must not erase the link the
 * marketing team already uses.
 */
export interface RegistryShot {
    schema: 1;
    scenario: string;
    screen: string;
    route: string;
    viewport: string;
    locale: string;
    width: number;
    height: number;
    fullPage: boolean;
    bytes: number;
    sha256: string;
    capturedAt: string;
    commit: string;
    baseURL: string;
    cdnBaseUrl: string;
    url: string | null;
    uploadedSha256: string | null;
    uploadedAt: string | null;
}

/** A published capture. */
export interface UploadInfo {
    url: string;
    sha256: string;
    uploadedAt: string;
}

/** Directory of the registry inside a checkout. */
export function registryDir(repoRoot: string): string {
    return join(repoRoot, "docs", "ui-screenshots", "shots");
}

/** File name of a registry entry — one file per capture, no shared files. */
export function registryFileName(entry: { scenario: string; screen: string; viewport: string; locale: string }): string {
    return `${entry.scenario}--${entry.screen}--${entry.viewport}--${entry.locale}.json`;
}

/** Reads every entry of the registry. */
export async function readRegistry(dir: string): Promise<RegistryShot[]> {
    let files: string[];
    try {
        files = (await readdir(dir)).filter((name) => name.endsWith(".json"));
    } catch {
        return [];
    }
    const entries: RegistryShot[] = [];
    for (const file of files) {
        try {
            entries.push(JSON.parse(await readFile(join(dir, file), "utf-8")) as RegistryShot);
        } catch {
            // A damaged entry must not break the whole registry.
        }
    }
    return entries;
}

/**
 * Writes a capture into the registry, replacing the previous entry of the
 * same screen.
 *
 * The upload facts are kept when the new capture was not published — the
 * published picture stays reachable until a new one replaces it.
 */
export async function mergeEntry(
    dir: string,
    entry: ShotEntry,
    upload?: UploadInfo,
): Promise<RegistryShot> {
    const key = shotKey(entry);
    const previous = (await readRegistry(dir)).find((candidate) => shotKey(candidate) === key);
    const merged: RegistryShot = {
        schema: 1,
        scenario: entry.scenario,
        screen: entry.screen,
        route: entry.route,
        viewport: entry.viewport,
        locale: entry.locale,
        width: entry.width,
        height: entry.height,
        fullPage: entry.fullPage,
        bytes: entry.bytes,
        sha256: entry.sha256,
        capturedAt: entry.capturedAt,
        commit: entry.commit,
        baseURL: entry.baseURL,
        cdnBaseUrl: entry.cdnBaseUrl,
        url: upload?.url ?? previous?.url ?? null,
        uploadedSha256: upload?.sha256 ?? previous?.uploadedSha256 ?? null,
        uploadedAt: upload?.uploadedAt ?? previous?.uploadedAt ?? null,
    };
    await mkdir(dir, { recursive: true });
    await writeFile(
        join(dir, registryFileName(merged)),
        `${JSON.stringify(merged, null, 2)}\n`,
        "utf-8",
    );
    return merged;
}
