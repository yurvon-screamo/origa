import type { Page } from "@playwright/test";

import { acquaintanceHand } from "./acquaintance-hand.ts";
import { counterDue } from "./counter-due.ts";
import { dueCorpus } from "./due-corpus.ts";
import { dueExamples } from "./due-examples.ts";
import { fresh } from "./fresh.ts";
import { irodoriNyuumon } from "./irodori-nyuumon.ts";
import { onboarded } from "./onboarded.ts";
import { onboardedInsights } from "./onboarded-insights.ts";
import type { PresetId } from "./registry.ts";
import { scoringN5 } from "./scoring-n5.ts";
import { withWord } from "./with-word.ts";

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
    "acquaintance-hand": acquaintanceHand,
    "counter-due": counterDue,
    "due-corpus": dueCorpus,
    "due-examples": dueExamples,
    onboarded,
    "scoring-n5": scoringN5,
    "onboarded-insights": onboardedInsights,
    "irodori-nyuumon": irodoriNyuumon,
    "with-word": withWord,
};
