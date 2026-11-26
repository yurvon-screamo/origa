import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** The public login screen — captured without a session. */
export const loginScenario: ShotScenario = {
  ...descriptors["login"],
  async run({ page }, sink) {
    await page.goto("/login");
    await expect(page.getByTestId("login-page")).toBeVisible({
      timeout: 60_000,
    });
    await sink.shot("login");
  },
};
