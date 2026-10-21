import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The insight panels of the home screen with live verdicts.
 *
 * The profile behind the screen is a seeded typical user (see the
 * `onboarded-insights` preset).
 */
export const homeInsightsScenario: ShotScenario = {
    ...descriptors["home-insights"],
    async run({ page }, sink) {
        await expect(page.getByTestId("home-insights-week-verdict")).toBeVisible({
            timeout: 120_000,
        });
        // Let fonts/images settle, then capture the viewport.
        await page.waitForTimeout(1_500);
        await sink.shot("insights");
    },
};
