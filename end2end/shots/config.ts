import { existsSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

/**
 * Shared decisions of the screenshot tool.
 *
 * The Playwright config imports this module, so it must stay free of
 * side effects: only `node:*` imports, never `../config` (its dotenv
 * bootstrap would run inside the Playwright process).
 */

/** How the app bundle is served. */
export type ServeMode = "build" | "dist";

/**
 * Single predicate for the serve mode.
 *
 * The Playwright config and the preflight MUST agree on it: if the
 * preflight checked the prebuilt bundle while the config built from
 * source, the font check would be bypassed exactly where it matters
 * (a local run builds through trunk, so `cdn/fonts/` is required).
 */
export const SERVE_MODE: ServeMode = process.env.CI ? "dist" : "build";

/** Production CDN endpoint — read-only, safe as a content source. */
export const DEFAULT_PROD_CDN_URL = "https://s3.origa.uwuwu.net";

/** Port the local corpus server listens on (the app points at it). */
export const LOCAL_CDN_PORT = 8080;

/** Port the app is served on. */
export const APP_PORT = 1420;

/** Port TrailBase listens on. */
export const TRAILBASE_PORT = 4000;

/** Repository root — `end2end/` lives one level below it. */
export const repoRoot = join(fileURLToPath(new URL("../..", import.meta.url)));

/** Where the CDN corpus lives by default (gitignored — see `.gitignore`). */
export const defaultCdnCacheDir = join(repoRoot, "cdn");

export interface CdnSource {
    /** URL the app bundle must point at for CDN content. */
    baseUrl: string;
    /** Local corpus directory; `null` when the content comes from the remote CDN. */
    cacheDir: string | null;
    /** Whether a local static server for the corpus has to be started. */
    serveLocally: boolean;
}

/**
 * Resolves where the app takes its CDN content from.
 *
 * A local corpus wins (same bytes the developer sees). Without one — a
 * fresh factory worktree has no `cdn/`, it is gitignored — the production
 * CDN serves the content instead. The URL is compiled into the bundle
 * (`build.rs` → `rustc-env`), so it is a build-time decision.
 */
export function resolveCdnSource(env: NodeJS.ProcessEnv = process.env): CdnSource {
    const cacheDir = env.ORIGA_CDN_CACHE_DIR ?? defaultCdnCacheDir;
    if (existsSync(cacheDir)) {
        return { baseUrl: `http://localhost:${LOCAL_CDN_PORT}`, cacheDir, serveLocally: true };
    }
    return {
        baseUrl: env.SHOT_CDN_BASE_URL ?? DEFAULT_PROD_CDN_URL,
        cacheDir: null,
        serveLocally: false,
    };
}
