import { skipOnboarding } from "../../helpers/navigation.ts";
import { isOnboardingShown } from "../../helpers/onboarding.ts";
import type { PresetContext } from "./index.ts";

/**
 * A fresh profile with an empty corpus.
 *
 * The onboarding wizard is skipped, so the app lands on the home screen
 * with nothing studied — the state a first-time user sees.
 */
export async function fresh(context: PresetContext): Promise<void> {
    if (await isOnboardingShown(context.page)) {
        await skipOnboarding(context.page);
    }
}
