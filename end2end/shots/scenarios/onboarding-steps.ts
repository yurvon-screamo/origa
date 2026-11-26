import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** The onboarding wizard steps, captured before any import starts. */
export const onboardingStepsScenario: ShotScenario = {
  ...descriptors["onboarding-steps"],
  async run({ page }, sink) {
    await expect(page.getByTestId("onboarding-intro-step")).toBeVisible({
      timeout: 60_000,
    });
    await sink.shot("step-intro");

    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-load-step")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("step-load");

    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible({
      timeout: 15_000,
    });
    await page.getByTestId("jlpt-option-n5").click();
    await sink.shot("step-jlpt");

    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-apps-step")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("step-apps");

    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-progress-step")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("step-progress");

    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-summary-step")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("step-summary");
  },
};
