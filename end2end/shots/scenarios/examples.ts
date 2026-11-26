import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** The examples list of the onboarded profile. */
export const examplesScenario: ShotScenario = {
  ...descriptors["examples"],
  async run({ page }, sink) {
    await page.goto("/examples");
    await expect(page.getByTestId("examples-page")).toBeVisible({
      timeout: 60_000,
    });
    await sink.shot("examples-list");
  },
};
