import { onboarded } from "./onboarded.ts";
import { readStoredUser, seedTypicalUserState, writeStoredUser } from "../seed.ts";
import type { PresetContext } from "./index.ts";

/**
 * A profile with the N4 corpus AND the typical active-user state.
 *
 * The onboarding gives the corpus (the cards behind the counters and the
 * troubled-words list), the seed adds three weeks of history, the
 * insight buckets and the lapses.
 */
export async function onboardedInsights(context: PresetContext): Promise<void> {
    await onboarded(context);

    const stored = await readStoredUser(context.page);
    await writeStoredUser(context.page, stored.key, seedTypicalUserState(stored.value));
    await context.page.reload({ waitUntil: "domcontentloaded" });
}
