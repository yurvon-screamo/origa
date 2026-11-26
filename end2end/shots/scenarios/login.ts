import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { captureSimplePage } from "./simple-page.ts";

/** The public login screen — captured without a session. */
export const loginScenario: ShotScenario = {
  ...descriptors["login"],
  async run({ page }, sink) {
    await captureSimplePage(page, sink, "/login", "login-page", "login");
  },
};
