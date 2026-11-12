import { gotoWordsRoute } from "../words-route.ts";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The word detail page of a translated word.
 *
 * The page shows the dictionary entry with its examples — the screen the
 * example-sentence feature (issue #528) landed on.
 */
export const wordExamplesScenario: ShotScenario = {
  ...descriptors["word-examples"],
  async run({ page }, sink) {
    await gotoWordsRoute(
      page,
      `/words/${encodeURIComponent(process.env.SHOT_WORD ?? "あさ")}`,
      "word-detail-word",
    );
    await page.waitForTimeout(3_000); // examples chunk fetch
    await sink.shot("detail-full", { fullPage: true });
    await sink.shot("detail-viewport");
  },
};
