import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** First-run consent: logged in, with the pre-approval script withheld. */
export const consentScenario: ShotScenario = {
  ...descriptors["consent"],
  async run({ page }, sink) {
    await expect(
      page.getByTestId("resource-consent-download-button"),
    ).toBeVisible({ timeout: 60_000 });
    // The consent screen mounts while the boot splash is still fading.
    // A frame taken underneath it shows a dimmed screen, not consent.
    await page
      .locator("#origa-boot-splash")
      .waitFor({ state: "detached", timeout: 60_000 });
    await sink.shot("consent-screen");
  },
};
