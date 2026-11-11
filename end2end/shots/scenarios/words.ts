import { gotoWordsRoute } from "../words-route.ts";
import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The words screen (vocabulary list).
 *
 * The header of this screen (title + tab strip + add button) is the
 * narrow-viewport regression surface: the capture pair proves the title
 * and the tabs stay a deliberate grid (title row, full-width tab strip)
 * instead of an accidental wrap.
 */
export const wordsScenario: ShotScenario = {
  ...descriptors.words,
  async run({ page }, sink) {
    await gotoWordsRoute(page, "/words");
    await page.waitForTimeout(3_000); // vocabulary chunk fetch
    await sink.shot("words");
    await sink.shot("words-full", { fullPage: true });
  },
};
