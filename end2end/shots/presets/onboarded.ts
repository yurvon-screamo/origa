import { completeOnboardingToScoring, isOnboardingShown } from "../../helpers/onboarding.ts";
import type { PresetContext } from "./index.ts";

/**
 * A profile with the whole N4 corpus imported and every card marked as
 * known.
 *
 * The corpus gives the screens real content (counters, insights, a due
 * queue) instead of empty states.
 */
export async function onboarded(context: PresetContext): Promise<void> {
    if (!(await isOnboardingShown(context.page))) return;

    await completeOnboardingToScoring(context.page, { level: "N4" });

    const markAll = context.page.getByTestId("onboarding-mark-all-known");
    if (await markAll.isVisible({ timeout: 10_000 }).catch(() => false)) {
        await markAll.click();
        await context.page
            .getByTestId("onboarding-confirm-ok")
            .click({ timeout: 10_000 })
            .catch(() => undefined);
    }
    await context.page
        .getByTestId("scoring-step-complete")
        .waitFor({ state: "visible", timeout: 120_000 });

    const { OnboardingPage } = await import("../../pages/onboarding.page.ts");
    await new OnboardingPage(context.page).clickFinish();
    await context.page.waitForURL(/\/home$/, { timeout: 300_000 });
}
