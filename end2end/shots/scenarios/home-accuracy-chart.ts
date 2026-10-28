import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The home accuracy chart (fl2O037UR7) with live data.
 *
 * The chart sits below the status cards, so the capture scrolls it into
 * view: the shot frames the chart with the tail of the status card above
 * it. (A full-page shot is useless here — the seeded profile marks the
 * whole corpus studied today, so StudiedTodayList alone is tens of
 * thousands of pixels tall.) On a narrow viewport the fixed tab bar
 * covers the bottom of the frame — it is hidden for the capture.
 *
 * The profile behind the screen is a seeded typical user (see the
 * `onboarded-history` preset) — its daily history draws the line.
 */
export const homeAccuracyChartScenario: ShotScenario = {
    ...descriptors["home-accuracy-chart"],
    async run({ page }, sink) {
        const chart = page.getByTestId("home-accuracy-chart-svg");
        await expect(chart).toBeVisible({ timeout: 120_000 });

        // Frame the whole section: its title at the top of the viewport,
        // the full card below (minimal scroll would cut the card).
        await page.getByTestId("home-accuracy-chart").evaluate((el) => {
            const top = el.getBoundingClientRect().top + window.scrollY - 24;
            window.scrollTo({ top });
        });

        // The home mount fires a sync toast that parks over the chart on
        // a narrow viewport — let it dismiss itself (5 s lifetime) before
        // framing the shot.
        await expect(page.locator('[data-testid^="toast-"]')).toHaveCount(0, {
            timeout: 15_000,
        });
        await page.waitForTimeout(500);

        const viewport = page.viewportSize();
        if (viewport && viewport.width < 500) {
            await page.addStyleTag({ content: ".bottom-tab-bar { display: none !important; }" });
            await page.waitForTimeout(300);
        }
        await sink.shot("accuracy-chart");
    },
};
