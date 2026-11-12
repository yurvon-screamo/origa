import { config } from "dotenv";
import { resolve } from "path";

import { end2endRoot } from "./helpers/paths";
import { APP_PORT, TRAILBASE_PORT } from "./shots/config.ts";

// Override system env vars with local .env values
config({ path: resolve(end2endRoot, ".env"), override: true });

export function getTrailBaseUrl(): string {
  // Single source of truth with the TrailBase spawn (global-setup):
  // both derive from the same E2E_TRAILBASE_PORT-driven port, so a
  // non-default port cannot desync the readiness probe / admin login
  // from the spawned server (bug report, test round 2026-11-11 — the
  // probe used to follow the .env TRAILBASE_URL line while the spawn
  // followed the env port). The .env TRAILBASE_URL line is no longer
  // consulted: dotenv's override:true made shell-level sync impossible
  // anyway.
  return `http://127.0.0.1:${TRAILBASE_PORT}`;
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
  return process.env.E2E_APP_BASE_URL ?? `http://localhost:${APP_PORT}`;
}
