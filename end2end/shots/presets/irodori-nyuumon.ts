import { isOnboardingShown } from "../../helpers/onboarding.ts";
import type { PresetContext } from "./index.ts";

/**
 * A profile with the Irodori Nyuumon lessons only.
 *
 * The JFT-Basic readiness card reads the Irodori catalog and the
 * Nyuumon progress, so the import consists of the Irodori sets — no JLPT
 * level is selected and the app step picks Irodori with the ninth lesson.
 */
export async function irodoriNyuumon(context: PresetContext): Promise<void> {
    const page = context.page;
    if (!(await isOnboardingShown(page))) return;

    // A fresh app origin has no resource cache: the first-run consent
    // overlay (or its auto-start download) may block the flow for a while.
    await page
        .getByTestId("resource-consent-download-button")
        .click({ timeout: 20_000 })
        .catch(() => undefined);
    await page
        .getByTestId("onboarding-spinner")
        .waitFor({ state: "hidden", timeout: 240_000 })
        .catch(() => undefined);

    await page.getByTestId("onboarding-next").click({ timeout: 240_000 });
    await page.getByTestId("onboarding-load-step").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByTestId("onboarding-next").click();
    await page.getByTestId("onboarding-jlpt-step").waitFor({ state: "visible", timeout: 10_000 });
    // No JLPT level selected: the import consists of the Irodori sets only.
    await page.getByTestId("onboarding-next").click();
    await page.getByTestId("onboarding-apps-step").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByTestId("apps-step-app-Irodori-checkbox").click();
    await page.getByTestId("onboarding-next").click();
    await page.getByTestId("onboarding-progress-step").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByTestId("irodori-book-dropdown").click();
    await page.getByTestId("irodori-book-dropdown-option-nyuumon").click();
    await page.getByTestId("irodori-lesson-dropdown").click();
    await page.getByTestId("irodori-lesson-dropdown-option-lesson_9").click();
    await page.getByTestId("onboarding-next").click();
    await page.getByTestId("onboarding-summary-step").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByTestId("onboarding-import").click();
    await page.getByTestId("onboarding-scoring-step").waitFor({ state: "visible", timeout: 240_000 });
    await page.getByTestId("scoring-step-hint").waitFor({ state: "visible", timeout: 60_000 });
    await page.getByTestId("onboarding-mark-all-known").click();
    await page.getByTestId("onboarding-confirm-ok").click();
    await page.getByTestId("scoring-step-complete").waitFor({ state: "visible", timeout: 240_000 });
    await page.getByTestId("onboarding-finish").click();
    await page.waitForURL(/\/home$/, { timeout: 240_000 });
}
