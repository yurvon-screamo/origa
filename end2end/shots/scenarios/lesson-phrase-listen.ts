import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/** A phrase card in the listen mode — the audio player is the discriminator. */
export const lessonPhraseListenScenario: ShotScenario = {
  ...descriptors["lesson-phrase-listen"],
  async run({ page }, sink) {
    for (let attempt = 0; attempt < 8; attempt++) {
      await page.goto("/lesson");
      const player = page.getByTestId("phrase-audio-player");
      if (
        await player
          .waitFor({ state: "visible", timeout: 90_000 })
          .then(() => true)
          .catch(() => false)
      ) {
        await page.waitForTimeout(700);
        await sink.shot("phrase-listen-question");
        return;
      }
      await page.reload({ waitUntil: "domcontentloaded" });
    }
    throw new Error("no phrase-listen view in the bounded reroll budget");
  },
};
