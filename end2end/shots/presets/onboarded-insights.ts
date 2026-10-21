import { onboarded } from "./onboarded.ts";
import { seedTypicalUserState } from "../seed.ts";
import type { PresetContext } from "./index.ts";
import { withSeededUser } from "./seeded.ts";

/**
 * A profile with the N4 corpus AND the typical active-user state.
 *
 * The onboarding gives the corpus (the cards behind the counters and the
 * troubled-words list), the seed adds three weeks of history, the
 * insight buckets and the lapses.
 */
export async function onboardedInsights(context: PresetContext): Promise<void> {
    await onboarded(context);
    await withSeededUser(context, seedTypicalUserState);
}
