/**
 * The known data states of a capture.
 *
 * Kept apart from the preset implementations: the catalog and the CLI
 * read the ids, the Playwright side owns the behaviour. A typo in an id
 * is a compile error here and a runtime error in the preset map.
 */
export const PRESET_IDS = [
    "acquaintance-hand",
    "due-corpus",
    "due-examples",
    "fresh",
    "onboarded",
    "scoring-n5",
    "onboarded-insights",
    "ledger",
    "irodori-nyuumon",
    "with-word",
] as const;

export type PresetId = (typeof PRESET_IDS)[number];
