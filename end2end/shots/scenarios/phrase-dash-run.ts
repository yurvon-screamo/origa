import type { Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import { waitForAppReady, waitForSyncToast } from "./simple-page.ts";
import type { ShotScenario } from "../scenario.ts";

/** The sanitized question text of the prod phrase behind #914. */
const QUESTION_TEXT = "実は少し";

/** Dash runs are dirt from the CDN chunk; the question must be clean. */
const DASH_RUN = /[─―—]{2,}/u;

/** The lesson places the phrase after its anchor word; a bounded walk is enough. */
const MAX_CARDS = 8;

/** A fresh lesson entry regenerates the shuffled core (same pattern as lesson-quiz-multi). */
const LESSON_ATTEMPTS = 3;

/**
 * Recognizes the phrase question front on the live lesson card.
 *
 * Both class selectors are deliberate: the question wrapper and the ruby
 * segments carry no test id in lesson_card_question.rs / furigana_hover.rs.
 */
async function phraseQuestionSeen(page: Page): Promise<boolean> {
  await page
    .getByTestId("lesson-card-root")
    .waitFor({ state: "visible", timeout: 1_000 })
    .catch(() => false);

  const root = page.getByTestId("lesson-card-root");
  if (
    await page
      .getByTestId("phrase-audio-player")
      .isVisible()
      .catch(() => false)
  ) {
    throw new Error(
      "phrase-dash-run: PhraseListen showed instead of the question front (#1760)",
    );
  }

  // No test id on the question wrapper; the furigana ruby splits the text
  // with rt/rp nodes, so compare the base text without them.
  const question = root.locator(".lesson-question");
  if (!(await question.isVisible().catch(() => false))) return false;
  const baseText = await question.evaluate((element) => {
    const clone = element.cloneNode(true) as HTMLElement;
    clone.querySelectorAll("rt, rp").forEach((node) => node.remove());
    return (clone.textContent ?? "").trim();
  });
  if (baseText !== QUESTION_TEXT) return false;

  await question
    .locator(".furigana-ruby")
    .first()
    .waitFor({ state: "visible", timeout: 120_000 });
  const rootText = (await root.textContent()) ?? "";
  if (DASH_RUN.test(rootText)) {
    throw new Error(
      "phrase-dash-run: dirty dash run rendered on the lesson question",
    );
  }
  return await page.getByTestId("lesson-show-answer-btn").isVisible();
}

/** Advance a non-target normal card with Good; Again would re-queue it. */
async function advanceWordCard(page: Page): Promise<boolean> {
  const reveal = page.getByTestId("lesson-show-answer-btn");
  if (await reveal.isVisible().catch(() => false)) await reveal.click();
  const good = page.getByTestId("lesson-rating-btn-good");
  const visible = await good
    .waitFor({ state: "visible", timeout: 3_000 })
    .then(() => true)
    .catch(() => false);
  if (!visible) return false;
  await good.click();
  // Rating saves asynchronously; the next card mounts after the tick.
  await page.waitForTimeout(700);
  return true;
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
    await waitForAppReady(page);
    await waitForSyncToast(page);
    await sink.shot("phrases-list-clean");

    // ── Surface 2: the lesson question card (the bug's surface) ──
    const { completeAcquaintanceHandIfPresent } = await import(
      "../../helpers/lesson.ts"
    );
    let found = false;
    for (let attempt = 0; attempt < LESSON_ATTEMPTS && !found; attempt++) {
      await page.goto("/lesson");
      await page
        .getByTestId("lesson-page")
        .waitFor({ state: "visible", timeout: 90_000 });
      await completeAcquaintanceHandIfPresent(page);
      for (let card = 0; card < MAX_CARDS; card++) {
        if (await phraseQuestionSeen(page)) {
          found = true;
          break;
        }
        if (
          await page
            .getByTestId("lesson-complete-screen")
            .isVisible()
            .catch(() => false)
        )
          break;
        if (!(await advanceWordCard(page))) break;
      }
    }
    if (!found) {
      throw new Error(
        "phrase-dash-run: the phrase question front never showed in the lesson walk",
      );
    }
    await waitForAppReady(page);
    await waitForSyncToast(page);
    await sink.shot("lesson-phrase-question-clean");
  },
};
