import { config } from "dotenv";
import { resolve } from "path";

import { end2endRoot } from "./helpers/paths";

// Override system env vars with local .env values
config({ path: resolve(end2endRoot, ".env"), override: true });

export function getTrailBaseUrl(): string {
    const url = process.env.TRAILBASE_URL;
    if (!url) {
        throw new Error("TRAILBASE_URL is not set. Configure it in .env file.");
    }
    return url;
}

/**
 * Base URL of the app under test.
 *
 * Deliberately NOT `ORIGA_APP_BASE_URL`: the repository prescribes that
 * variable as the PRODUCTION app URL (`https://app.origa.uwuwu.net`), and
 * the e2e stack must never navigate to production — a test account would
 * land in the production TrailBase. Read at call time, so the dotenv
 * bootstrap above has already run.
 */
export function getAppBaseUrl(): string {
    return process.env.E2E_APP_BASE_URL ?? "http://localhost:1420";
}
