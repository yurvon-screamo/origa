import { onboarded } from "./onboarded.ts";
import { seedLedgerState } from "../seed.ts";
import type { PresetContext } from "./index.ts";
import { withSeededUser } from "./seeded.ts";

/**
 * A profile with the demo corpus of the insight ledger.
 *
 * The corpus of the demo set is written on top of an imported one: the
 * insight panel needs a real knowledge set behind the verdicts, and the
 * demo words carry the review states the ledger shows.
 */
export async function ledger(context: PresetContext): Promise<void> {
    await onboarded(context);
    await withSeededUser(context, seedLedgerState);
}
