import { completeOnboardingToScoring, isOnboardingShown } from "../../helpers/onboarding.ts";
import type { PresetContext } from "./index.ts";

/**
 * A fresh profile stopped at the interactive assessment.
 *
 * The N5 corpus is imported, but the assessment is NOT finished: the
 * scenario walks the card queue itself, card by card.
 */
export async function scoringN5(context: PresetContext): Promise<void> {
    if (!(await isOnboardingShown(context.page))) return;

    const reached = await completeOnboardingToScoring(context.page, {
        level: "N5",
        onboardingUrlTimeout: 420_000,
        scoringReadyTimeout: 600_000,
    });
    if (!reached) {
        throw new Error("scoring step must be reachable for a fresh N5 user");
    }
}
