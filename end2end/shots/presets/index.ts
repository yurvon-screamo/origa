import type { Page } from "@playwright/test";

import { fresh } from "./fresh.ts";
import type { PresetId } from "./registry.ts";

/** What a preset works with. */
export interface PresetContext {
    page: Page;
    baseURL: string;
}

/**
 * A preset prepares the app state a scenario expects.
 *
 * The set is open: a new state is a new function, not a new variant of a
 * closed union — the onboarding flows of the existing specs differ enough
 * (a plain skip, an import with a single app, a seeded corpus) that a
 * closed type would grow a case per pull request.
 */
export type Preset = (context: PresetContext) => Promise<void>;

export const presets: Record<PresetId, Preset> = {
    fresh,
};
