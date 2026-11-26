import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { captureSimplePage } from "./simple-page.ts";

/** The empty word list of a fresh profile. */
export const listsEmptyScenario: ShotScenario = {
  ...descriptors["lists-empty"],
  async run({ page }, sink) {
    await captureSimplePage(
      page,
      sink,
      "/words",
      "words-empty-state",
      "words-empty",
    );
  },
};
