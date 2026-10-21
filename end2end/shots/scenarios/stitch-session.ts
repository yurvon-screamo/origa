import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The insight ledger of the home screen, whole page.
 *
 * The ledger sits below the counters, so the capture is a full-page one.
 * On a narrow viewport the fixed tab bar is stamped into the middle of a
 * full-page screenshot and covers the ledger rows — it is hidden for the
 * capture.
 */
export const stitchSessionScenario: ShotScenario = {
    ...descriptors["stitch-session"],
    async run({ page }, sink) {
        await expect(page.getByTestId("home-insights-week-verdict")).toBeVisible({
            timeout: 420_000,
        });
        await page.waitForTimeout(2_000);

        const viewport = page.viewportSize();
        if (viewport && viewport.width < 500) {
            await page.addStyleTag({ content: ".bottom-tab-bar { display: none !important; }" });
            await page.waitForTimeout(300);
        }
        await sink.shot("ledger", { fullPage: true });
    },
};
