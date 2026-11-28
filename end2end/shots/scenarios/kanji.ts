import { expect } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { waitForAppReady } from "./simple-page.ts";

/** The kanji list and its add drawer. */
export const kanjiScenario: ShotScenario = {
  ...descriptors["kanji"],
  async run({ page, viewport }, sink) {
    await page.goto("/kanji");
    await expect(page.getByTestId("kanji-page")).toBeVisible({
      timeout: 60_000,
    });
    await waitForAppReady(page);
    await sink.shot("kanji-list", { marker: "kanji-page" });
    await page.getByTestId("kanji-add-btn").click();
    await expect(page.getByTestId("kanji-add-drawer")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("kanji-add-drawer", { marker: "kanji-add-drawer" });

    await page.keyboard.press("Escape");
    await page.getByTestId("kanji-card-item").first().click();
    // The tab strip is the mobile navigation of the detail screen: the
    // desktop layout keeps the same block hidden and renders the panes
    // inline. Capture it only where the user can actually see it.
    if (viewport === "mobile") {
      await expect(page.getByTestId("kanji-detail-tabs")).toBeVisible({
        timeout: 30_000,
      });
      await sink.shot("kanji-detail-tabs", { marker: "kanji-detail-tabs" });
    }
  },
};
