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
    await page
      .getByTestId("phrases-card-item")
      .first()
      .waitFor({ state: "visible", timeout: 60_000 });
    await page.waitForTimeout(3_000);
    // Token surfaces carry no data-testid; pick a verified particle so the
    // capture shows the popup state added in uwuwu#938, not a noun popup.
    const particle = /^(は|が|を|に|で|と|の|か|よ|ね|も|から|まで|へ|や)$/;
    const token = page
      .locator(".token-word .token-surface")
      .filter({ hasText: particle })
      .first();
    await token.waitFor({ state: "visible", timeout: 30_000 });
    await token.click();
    await page
      .getByTestId("token-popup-report")
      .waitFor({ state: "visible", timeout: 10_000 });
    await sink.shot("token-popup-report", { marker: "token-popup-report" });
  },
};
