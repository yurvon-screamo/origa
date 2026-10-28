import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The home accuracy chart (fl2O037UR7) with live data.
 *
 * The chart sits below the status cards, so the capture is a full-page
 * one: the whole home shows at once — the reclaimed layout and the chart
 * in context. On a narrow viewport the fixed tab bar is stamped into the
 * middle of a full-page screenshot and covers the content — it is hidden
 * for the capture.
 *
 * The profile behind the screen is a seeded typical user (see the
 * `onboarded-insights` preset) — its daily history draws the line.
 */
export const homeAccuracyChartScenario: ShotScenario = {
    ...descriptors["home-accuracy-chart"],
    async run({ page }, sink) {
        await expect(page.getByTestId("home-accuracy-chart-svg")).toBeVisible({
            timeout: 120_000,
        });
        await page.waitForTimeout(1_500);

        const viewport = page.viewportSize();
        if (viewport && viewport.width < 500) {
            await page.addStyleTag({ content: ".bottom-tab-bar { display: none !important; }" });
            await page.waitForTimeout(300);
        }
        await sink.shot("accuracy-chart", { fullPage: true });
    },
};
