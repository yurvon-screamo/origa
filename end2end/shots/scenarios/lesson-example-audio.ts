import { type Locator, type Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import type { ShotSink } from "../types.ts";

const CARD_BUDGET = 4;
const RELOAD_BUDGET = 2;
const AUDIO_WAIT_MS = 45_000;
// The audio-driven question side shows this locale-pinned copy instead
// of the sentence (`example_listen` in locales/ru.json). The speaker
// button alone is not enough: text-mode cards render it too.
const LISTEN_HINT = "Прослушайте предложение";

async function visible(locator: Locator, timeout: number): Promise<boolean> {
  try {
    await locator.waitFor({ state: "visible", timeout });
    return true;
  } catch {
    return false;
  }
}

async function advance(page: Page): Promise<boolean> {
  const reveal = page.getByTestId("lesson-example-reveal");
  if (await visible(reveal, 2_000)) await reveal.click();
  for (const id of ["lesson-example-next", "lesson-example-understood"]) {
    const button = page.getByTestId(id);
    if (await visible(button, 2_000)) {
      await button.click();
      return true;
    }
  }
  return false;
}

/** Walks example cards until the audio-driven question side is shown. */
async function findAudioMode(page: Page, sink: ShotSink): Promise<boolean> {
  for (let card = 0; card < CARD_BUDGET; card++) {
    if (await visible(page.getByTestId("lesson-example-unavailable"), 2_000)) {
      if (!(await advance(page))) return false;
      continue;
    }
    const audio = page.getByTestId("lesson-example-audio");
    if (!(await visible(audio, AUDIO_WAIT_MS))) return false;
    const question = await page.getByTestId("lesson-example-ja").innerText();
    if (question.includes(LISTEN_HINT)) {
      await page.waitForTimeout(700);
      await sink.shot("example-audio-question");
      return true;
    }
    if (!(await advance(page))) return false;
  }
  return false;
}

/** The example card in audio mode: the sentence hides behind the speaker. */
export const lessonExampleAudioScenario: ShotScenario = {
  ...descriptors["lesson-example-audio"],
  async run({ page }, sink) {
    for (let attempt = 0; attempt < RELOAD_BUDGET; attempt++) {
      await page.goto("/lesson");
      await page
        .getByTestId("lesson-page")
        .waitFor({ state: "visible", timeout: 90_000 });
      if (await findAudioMode(page, sink)) return;
      await page.reload({ waitUntil: "domcontentloaded" });
    }
    throw new Error("no example audio-mode card in the bounded walk");
  },
};
