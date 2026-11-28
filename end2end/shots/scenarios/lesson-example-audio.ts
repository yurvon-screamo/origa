import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** An example-recall card whose audio button is present (premiere/audio view). */
export const lessonExampleAudioScenario: ShotScenario = {
  ...descriptors["lesson-example-audio"],
  async run({ page }, sink) {
    for (let attempt = 0; attempt < 4; attempt++) {
      await page.goto("/lesson");
      const card = page.getByTestId("lesson-example-recall-card");
      if (
        await card
          .waitFor({ state: "visible", timeout: 30_000 })
          .then(() => true)
          .catch(() => false)
      ) {
        const audio = page.getByTestId("lesson-example-audio");
        if (await audio.isVisible().catch(() => false)) {
          await page.waitForTimeout(700);
          await sink.shot("example-audio-question", {
            marker: "lesson-example-audio",
          });
          return;
        }
      }
      await page.reload({ waitUntil: "domcontentloaded" });
    }
    throw new Error("no audio example view in the bounded reroll budget");
  },
};
