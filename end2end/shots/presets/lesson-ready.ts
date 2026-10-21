import { completeOnboardingToScoring, isOnboardingShown } from "../../helpers/onboarding.ts";
import type { PresetContext } from "./index.ts";

/**
 * A profile with the corpus imported but the assessment skipped.
 *
 * Skipping the interactive assessment keeps every card NEW — the state a
 * lesson expects: the due cards exist and the lesson button is there.
 */
export async function lessonReady(context: PresetContext): Promise<void> {
    if (!(await isOnboardingShown(context.page))) return;

    await completeOnboardingToScoring(context.page, { level: "N4" });

    const skipButton = context.page.getByTestId("onboarding-skip-scoring");
    await skipButton.waitFor({ state: "visible", timeout: 90_000 }).catch(() => undefined);
    if (await skipButton.isVisible().catch(() => false)) {
        await skipButton.click();
        await context.page
            .getByTestId("onboarding-confirm-ok")
            .waitFor({ state: "visible", timeout: 10_000 })
            .catch(() => undefined);
        await context.page
            .getByTestId("onboarding-confirm-ok")
            .click({ timeout: 10_000 })
            .catch(() => undefined);
    }

    const { OnboardingPage } = await import("../../pages/onboarding.page.ts");
    const onboarding = new OnboardingPage(context.page);
    const finishVisible = await onboarding.finishButton
        .waitFor({ state: "visible", timeout: 90_000 })
        .then(() => true)
        .catch(() => false);
    if (finishVisible) {
        await onboarding.clickFinish();
    }
    await context.page.waitForURL(/\/home$/, { timeout: 300_000 });
}
