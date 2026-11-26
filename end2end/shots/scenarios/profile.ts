import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** The profile page — personal data and the settings card. */
export const profileScenario: ShotScenario = {
  ...descriptors["profile"],
  async run({ page }, sink) {
    await page.goto("/profile");
    await expect(page.getByTestId("profile-page")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("profile-personal-data")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("profile-form");
    await expect(page.getByTestId("profile-settings")).toBeVisible();
    await sink.shot("profile-settings");
  },
};
