import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { OnboardingPage } from "../../pages";
import { completeOnboardingToScoring } from "../../helpers/onboarding";

// Regression: pressing "Start Import" with NO sets selected used to log a
// warning and strand the user on the summary step with a silent UI. Empty
// selection is a valid outcome — the app must persist the earlier-step
// choices and advance to scoring (an empty scoring queue completes
// immediately, so onboarding finishes from there).

Given('новый пользователь дошёл до шага сводки импорта, не выбрав ни одного сета', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    const reached = await completeOnboardingToScoring(page, {
        level: "none",
        skipApps: true,
        stopAtSummary: true,
    });
    expect(reached, "onboarding must stop at the summary step").toBe(true);
    await expect(onboarding.summaryStep).toBeVisible();
});

When('пользователь нажимает кнопку импорта', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    // No data-loading assertion here: the empty-import path skips tokenizing
    // and corpus download, so the loading window can close before the first
    // Playwright poll — asserting it would make this step flaky.
    await onboarding.startImport();
});

Then('отображается шаг оценивания с пустым списком карточек', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await expect(onboarding.scoringStep).toBeVisible({ timeout: 30_000 });
    // Zero imported sets → zero scoring cards → the step reports completion
    // right away (scoring_step.rs sets scoring_completed when total == 0).
    await expect(onboarding.scoringComplete).toBeVisible({ timeout: 60_000 });
});
