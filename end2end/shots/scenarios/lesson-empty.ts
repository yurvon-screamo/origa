import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { captureSimplePage } from "./simple-page.ts";

/** The lesson page with no due cards and no acquaintance hand. */
export const lessonEmptyScenario: ShotScenario = {
  ...descriptors["lesson-empty"],
  async run({ page }, sink) {
    await captureSimplePage(
      page,
      sink,
      "/lesson",
      "lesson-empty-import-btn",
      "lesson-empty",
    );
  },
};
