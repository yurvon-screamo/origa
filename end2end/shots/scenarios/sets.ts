import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { waitForAppReady } from "./simple-page.ts";

/** Sets: level filter and the import preview drawer. */
export const setsScenario: ShotScenario = {
  ...descriptors["sets"],
  async run({ page }, sink) {
    await page.goto("/sets");
    await expect(page.getByTestId("sets-page")).toBeVisible({
      timeout: 60_000,
    });
    await waitForAppReady(page);
    await sink.shot("sets-list");
    await page.getByTestId("sets-level-n5").click();
    await sink.shot("sets-filters");
    await page.getByTestId("sets-import-new").click();
    await page.getByTestId("sets-card-item").first().click();
    await page.getByTestId("sets-import-selected-btn").click();
    await expect(page.getByTestId("sets-import-drawer")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("sets-import-found");
  },
};
