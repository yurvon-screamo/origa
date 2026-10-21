import { createHash } from "node:crypto";

import type { ViewportTag } from "./viewports.ts";

/**
 * One captured screen.
 *
 * The entry is the contract between the capture process and the registry:
 * it carries the image facts (size, hash) AND the provenance (which
 * scenario, which build, which content source) — a screenshot without
 * provenance cannot be reproduced.
 */
export interface ShotEntry {
    scenario: string;
    title: string;
    screen: string;
    route: string;
    viewport: ViewportTag;
    locale: string;
    width: number;
    height: number;
    fullPage: boolean;
    bytes: number;
    sha256: string;
    file: string;
    capturedAt: string;
    commit: string;
    baseURL: string;
    cdnBaseUrl: string;
}

export interface ShotManifest {
    version: 1;
    label: string;
    entries: ShotEntry[];
}

/** SHA-256 of a buffer, hex encoded. */
export function sha256Hex(data: Buffer): string {
    return createHash("sha256").update(data).digest("hex");
}

/**
 * Pixel dimensions of a PNG, read from the IHDR chunk.
 *
 * The capture does not trust the requested viewport: the file on disk is
 * the fact that ends up in the registry.
 */
export function readPngSize(data: Buffer): { width: number; height: number } {
    return { width: data.readUInt32BE(16), height: data.readUInt32BE(20) };
}

/** File name of a capture — deterministic, one screen per file. */
export function shotFileName(scenario: string, screen: string, viewport: ViewportTag): string {
    return `${scenario}--${screen}--${viewport}.png`;
}

/** Key of a capture inside the registry. */
export function shotKey(entry: ShotEntry): string {
    return `${entry.scenario}--${entry.screen}--${entry.viewport}--${entry.locale}`;
}
