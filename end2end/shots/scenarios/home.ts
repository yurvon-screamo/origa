import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { waitForAppReady } from "./simple-page.ts";

/**
 * The home screen with an empty corpus.
 *
 * The first screen a user sees — the one every pull request that touches
 * the layout has to show.
 */
export const homeScenario: ShotScenario = {
  ...descriptors.home,
  async run({ page }, sink) {
    await page.goto("/home");
    await page
      .getByTestId("home-page")
      .waitFor({ state: "visible", timeout: 60_000 });
    await waitForAppReady(page);
    // The mount sync toast parks over the header for ~5 s; a baseline
    // frame with it still fading creates a false diff on the next PR.
    await expect(page.locator('[data-testid^="toast-"]')).toHaveCount(0, {
      timeout: 15_000,
    });
    await sink.shot("home");
  },
};
