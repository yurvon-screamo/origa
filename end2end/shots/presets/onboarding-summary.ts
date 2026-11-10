import {
  completeOnboardingToScoring,
  isOnboardingShown,
} from "../../helpers/onboarding.ts";
import type { PresetContext } from "./index.ts";

/**
 * A fresh profile stopped at the import summary step.
 *
 * The JLPT N3 selection (n5+n4+n3, ~3k words) is queued, the import has
 * NOT run: the scenario triggers it itself with a throttled CDN so the
 * progress line is observable.
 */
export async function onboardingSummary(context: PresetContext): Promise<void> {
  if (!(await isOnboardingShown(context.page))) return;

  const reached = await completeOnboardingToScoring(context.page, {
    level: "N3",
    skipApps: true,
    stopAtSummary: true,
    onboardingUrlTimeout: 420_000,
  });
  if (!reached) {
    throw new Error("summary step must be reachable for a fresh N3 user");
  }
}
