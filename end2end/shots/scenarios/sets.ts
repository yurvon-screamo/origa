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
    await sink.shot("sets-list", { marker: "sets-page" });
    await page.getByTestId("sets-level-n5").click();
    await sink.shot("sets-filters", { marker: "sets-level-n5" });
    await page.getByTestId("sets-import-new").click();
    // Selection lives on the card checkbox, not on the card root. A
    // root click leaves `selected_sets` empty, so the import button
    // never mounts and the unbounded click hangs the whole scenario.
    // The real input is visually hidden; the label around it is the
    // clickable surface. Clicking the card root does not select either.
    await page
      .getByTestId("sets-card-item")
      .first()
      .locator("label.checkbox-container")
      .click({ timeout: 15_000 });
    await expect(page.getByTestId("sets-import-selected-btn")).toBeVisible({
      timeout: 15_000,
    });
    await page.getByTestId("sets-import-selected-btn").click();
    await expect(page.getByTestId("sets-import-drawer")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("sets-import-found", { marker: "sets-import-drawer" });
  },
};
