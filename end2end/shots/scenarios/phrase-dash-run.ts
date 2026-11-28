import type { Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The prod phrase behind #914 on both client surfaces that render
 * `PhraseDetail.text`: the phrases list card and the lesson question
 * card.
 *
 * The `phrase-dash-run` preset seeds a fresh profile with the dirty
 * prod phrase (chunk p0024, `実は少し────`) as its only due card — the
 * chunk bytes stay dirty on the CDN, the load boundary must sanitize
 * them before any view reads the store. The lesson walk completes the
 * bootstrap hand (when the profile needed one) and rerolls on reloads
 * until the phrase question front shows: the lesson body view mix is
 * randomized, a reload regenerates it for free. A dirty render never
 * satisfies the front probe (the ──── run is rejected explicitly), so
 * the budget error doubles as the regression signal.
 */
const REROLL_BUDGET = 5;
const FRONT_WAIT_MS = 20_000;

/** Polls the page until the phrase question front is on screen, clean. */
async function phraseFrontSeen(page: Page): Promise<boolean> {
  const deadline = Date.now() + FRONT_WAIT_MS;
  while (Date.now() < deadline) {
    const probe = await page.evaluate(() => document.body.textContent ?? "");
    const showAnswer = await page
      .getByTestId("lesson-show-answer-btn")
      .isVisible()
      .catch(() => false);
    if (
      probe.includes("実") &&
      probe.includes("は") &&
      !probe.includes("────") &&
      showAnswer
    ) {
      return true;
    }
    await page.waitForTimeout(500);
  }
  return false;
}

export const phraseDashRunScenario: ShotScenario = {
  ...descriptors["phrase-dash-run"],
  async run({ page }, sink) {
    // ── Surface 1: the phrases list card ──
    await page.goto("/phrases");
    const cardText = page
      .getByTestId("phrases-card-item")
      .first()
      .getByTestId("phrases-card-text");
    await cardText.waitFor({ state: "visible", timeout: 60_000 });
    await page.waitForTimeout(1_500);
    await sink.shot("phrases-list-clean", { marker: "phrases-card-text" });

    // ── Surface 2: the lesson question card (the bug's surface) ──
    const { completeAcquaintanceHandIfPresent } = await import(
      "../../helpers/lesson.ts"
    );
    for (let attempt = 0; attempt < REROLL_BUDGET; attempt++) {
      await page.goto("/lesson");
      await page
        .getByTestId("lesson-page")
        .waitFor({ state: "visible", timeout: 90_000 });
      await completeAcquaintanceHandIfPresent(page);
      if (await phraseFrontSeen(page)) {
        await page.waitForTimeout(1_500);
        await sink.shot("lesson-phrase-question-clean", {
          marker: "lesson-page",
        });
        return;
      }
    }
    throw new Error(
      "the phrase question front never showed within the reroll budget",
    );
  },
};
