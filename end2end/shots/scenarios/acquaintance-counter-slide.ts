import { expect, type Page } from "@playwright/test";

import { runAcquaintancePresentation } from "../../helpers/lesson.ts";

import { descriptors } from "../catalog.ts";
import { readStoredUser, writeStoredUser } from "../seed.ts";
import type { ShotScenario } from "../scenario.ts";

/** A new counter card: the acquaintance hand only takes new cards. */
const COUNTER_CARD_ID = "01JCBZ8QK0AX4W7YH2Z5P9T3GE";

/** The acquaintance-hand preset seeds words, kanji and grammar, but no
 * counter card. Inject one into the same local record before the lesson
 * builds its hand — a counter slide is otherwise unreachable. */
async function seedNewCounterCard(page: Page): Promise<void> {
  const stored = await readStoredUser(page);
  const user = JSON.parse(stored.value) as {
    knowledge_set?: {
      study_cards?: Record<string, unknown>;
    };
  };
  user.knowledge_set ??= {};
  user.knowledge_set.study_cards ??= {};
  user.knowledge_set.study_cards[COUNTER_CARD_ID] = {
    card_id: COUNTER_CARD_ID,
    card: {
      Counter: {
        suffix: "本",
        bindings: [1, 2, 3, 0].map((number) => ({ number, memory: {} })),
      },
    },
    memory_history: {},
    is_favorite: false,
    perfect_streak_since_known: 0,
    favorite_changed_at: null,
  };
  await writeStoredUser(page, stored.key, JSON.stringify(user));
}

/** The counter slide of the acquaintance hand: presentation and answer. */
export const acquaintanceCounterSlideScenario: ShotScenario = {
  ...descriptors["acquaintance-counter-slide"],
  async run({ page }, sink) {
    await seedNewCounterCard(page);
    await page.goto("/lesson");
    await page
      .getByTestId("acquaintance-view")
      .waitFor({ state: "visible", timeout: 180_000 });

    const next = page.getByTestId("acquaintance-next-btn");
    let presentation = false;
    for (let index = 0; index < 12 && !presentation; index++) {
      if (
        await page
          .getByTestId("acquaintance-counter-slide")
          .isVisible()
          .catch(() => false)
      ) {
        await page.waitForTimeout(700);
        await sink.shot("counter-presentation");
        presentation = true;
        break;
      }
      await next.click({ timeout: 2_000 }).catch(() => undefined);
      await page.waitForTimeout(500);
    }
    expect(presentation, "the hand must show the counter slide").toBe(true);

    // The counter slide is not necessarily the last presentation slide.
    // The shared helper walks the rest of the presentation and asserts
    // that training is actually on screen.
    await runAcquaintancePresentation(page);

    let answer = false;
    for (let index = 0; index < 60 && !answer; index++) {
      if (await finished(page)) break;
      if (
        !(await page
          .getByTestId("acquaintance-training")
          .isVisible()
          .catch(() => false))
      ) {
        await page.waitForTimeout(800);
        continue;
      }
      const reveal = page.getByTestId("acquaintance-reveal-btn");
      if (await reveal.isVisible().catch(() => false)) {
        await reveal.click({ timeout: 3_000 }).catch(() => undefined);
        if (
          await page
            .getByTestId("acquaintance-counter-answer")
            .waitFor({ state: "visible", timeout: 4_000 })
            .then(() => true)
            .catch(() => false)
        ) {
          await page.waitForTimeout(700);
          await sink.shot("counter-training-answer");
          answer = true;
          break;
        }
      }
      // «Не помню» keeps the criterion open so the rotation reaches
      // the counter card. «Помню» closes the hand after three successes
      // and the counter answer is never shown.
      await page
        .getByTestId("acquaintance-rating-dont-know")
        .click({ timeout: 2_000 })
        .catch(() => undefined);
      await page.waitForTimeout(500);
    }
    expect(answer, "the hand must show the counter answer").toBe(true);
  },
};

function finished(page: Page): Promise<boolean> {
  return page
    .getByTestId("acquaintance-completed")
    .or(page.getByTestId("lesson-complete-screen"))
    .isVisible()
    .catch(() => false);
}
