import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import { waitForScoringReady } from "../../helpers/onboarding.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The live import progress on the summary step.
 *
 * The corpus bodies pay a simulated network round trip, so the phase
 * line under the import button (Загрузка наборов k/N…) stays visible
 * long enough to capture — and the pipeline still has to land on the
 * scoring step, which is the end-to-end proof the import completes.
 *
 * The progress budget is minutes, not seconds: on a debug WASM build the
 * tokenizer warmup (ensure_tokenizer_loaded inside the import callback)
 * can outlast a short wait, and until it concludes no phase event exists
 * by design.
 */
export const onboardingImportProgressScenario: ShotScenario = {
  ...descriptors["onboarding-import-progress"],
  async run({ page }, sink) {
    await page.route("**/well_known_set/**", async (route) => {
      await new Promise((resolve) => setTimeout(resolve, 800));
      await route.continue();
    });

    await page.getByTestId("onboarding-import").click();
    const progress = page.getByTestId("onboarding-import-progress");
    await expect(progress).toBeVisible({ timeout: 600_000 });

    await page.waitForTimeout(500);
    await sink.shot("progress");

    await waitForScoringReady(page, 600_000);
    await sink.shot("scoring");
  },
};
