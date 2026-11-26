import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { captureSimplePage } from "./simple-page.ts";

/** The examples list of the onboarded profile. */
export const examplesScenario: ShotScenario = {
  ...descriptors["examples"],
  async run({ page }, sink) {
    await captureSimplePage(
      page,
      sink,
      "/examples",
      "examples-page",
      "examples-list",
    );
  },
};
