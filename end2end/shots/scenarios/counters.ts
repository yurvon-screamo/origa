import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { waitForAppReady } from "./simple-page.ts";

/** The counters list and its add drawer. */
export const countersScenario: ShotScenario = {
  ...descriptors["counters"],
  async run({ page }, sink) {
    await page.goto("/counters");
    await expect(page.getByTestId("counters-page")).toBeVisible({
      timeout: 60_000,
    });
    await waitForAppReady(page);
    await sink.shot("counters-list", { marker: "counters-page" });
    await page.getByTestId("counters-add-btn").click();
    await expect(page.getByTestId("counters-add-drawer")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("counters-add-drawer", { marker: "counters-add-drawer" });
  },
};
