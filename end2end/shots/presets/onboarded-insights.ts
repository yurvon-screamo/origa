import { onboarded } from "./onboarded.ts";
import { seedTypicalUserState } from "../seed.ts";
import type { PresetContext } from "./index.ts";
import { withSeededUser } from "./seeded.ts";

/**
 * A profile with the N4 corpus AND the typical active-user state.
 *
 * The onboarding gives the corpus (the cards behind the counters), the
 * seed adds the month of daily ratings the accuracy chart draws and the
 * lapses behind the difficult counter.
 */
export async function onboardedInsights(context: PresetContext): Promise<void> {
    await onboarded(context);
    await withSeededUser(context, seedTypicalUserState);
}
