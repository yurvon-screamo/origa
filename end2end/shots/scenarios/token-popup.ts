import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The token popup of a phrase card with its report line.
 *
 * The phrases catalogue comes from the imported corpus, so the profile
 * behind the screen carries one (see the `onboarded` preset).
 */
export const tokenPopupScenario: ShotScenario = {
    ...descriptors["token-popup"],
    async run({ page }, sink) {
        await page.goto("/phrases");
        await page.getByTestId("phrases-card-item").first().waitFor({ state: "visible", timeout: 60_000 });
        await page.waitForTimeout(3_000);
        const token = page.locator(".token-word .token-surface").first();
        await token.waitFor({ state: "visible", timeout: 30_000 });
        await token.click();
        await page.getByTestId("token-popup-report").waitFor({ state: "visible", timeout: 10_000 });
        await sink.shot("token-popup-report");
    },
};
