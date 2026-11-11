import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The JLPT progress card with the category matrix expanded.
 *
 * The profile behind the screen imported the N4 corpus (see the
 * `onboarded` preset) — the matrix shows real per-category numbers.
 * The home scenario cannot cover this state: the matrix lives behind
 * the «Подробнее» disclosure.
 */
export const homeJlptDetailsScenario: ShotScenario = {
  ...descriptors["home-jlpt-details"],
  async run({ page }, sink) {
    await page.goto("/home");
    await page
      .getByTestId("home-page")
      .waitFor({ state: "visible", timeout: 60_000 });
    const toggle = page.getByTestId("home-jlpt-progress-toggle");
    await toggle.waitFor({ state: "visible", timeout: 60_000 });
    await toggle.click();
    await page
      .getByTestId("home-jlpt-progress-expanded-kanji")
      .waitFor({ state: "visible", timeout: 10_000 });
    await sink.shot("home-jlpt-details");
  },
};
