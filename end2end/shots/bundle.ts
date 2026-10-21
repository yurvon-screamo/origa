import { SERVE_MODE } from "./config.ts";

/** Bundle assets of the Leptos build, as referenced from index.html. */
const ASSET_PATTERN = /origa_ui-[^"'\\s]+\.(?:js|wasm)/g;

/** Provenance value for a capture whose content source cannot be proven. */
export const UNKNOWN_CDN = "unknown (dist)";

/**
 * The detected value of the run.
 *
 * The served bundle is the same for every scenario of a run, and the
 * detection downloads it — once per run is enough.
 */
let detected: { baseURL: string; expected: string; value: string } | null = null;

/**
 * Which CDN the served bundle actually points at.
 *
 * The URL is compiled into the bundle (`build.rs` → `rustc-env`), so in
 * build mode the expected value is the baked one. In dist mode the bundle
 * was built elsewhere: the served assets are inspected instead, and an
 * unprovable value is reported honestly rather than guessed.
 */
export async function detectCdnBaseUrl(baseURL: string, expected: string): Promise<string> {
    if (detected?.baseURL === baseURL && detected.expected === expected) {
        return detected.value;
    }
    const value = await readCdnBaseUrl(baseURL, expected);
    detected = { baseURL, expected, value };
    return value;
}

async function readCdnBaseUrl(baseURL: string, expected: string): Promise<string> {
    if (SERVE_MODE === "build") return expected;
    try {
        const html = await (await fetch(baseURL)).text();
        const assets = [...html.matchAll(ASSET_PATTERN)].map((match) => match[0]);
        for (const asset of assets) {
            const response = await fetch(new URL(asset, baseURL));
            if (!response.ok) continue;
            const body = await response.text();
            if (body.includes(expected)) return expected;
        }
    } catch {
        // The bundle is unreachable — the capture itself will fail loudly.
    }
    return UNKNOWN_CDN;
}
