import { expect, type Page } from "@playwright/test";

import { answerTrainingRemember } from "../../helpers/lesson.ts";
import { descriptors } from "../catalog.ts";
import { readStoredUser, writeStoredUser } from "../seed.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The word family of the acquaintance hand: presentation, both training
 * fronts and the answer side — with the audio button inside the card.
 *
 * Task #921 moved the word's audio button from the tags row above the
 * card INTO the card (top-right corner): the flex-wrap tags row used to
 * drop the button onto its own line on narrow viewports, floating above
 * the card, detached from it. The matrix proves the new home:
 * presentation and the text front show the corner button; the audio
 * front keeps its central replay button and NO corner button (no
 * doubles); the answer side brings the corner button back.
 *
 * The first seeded word is swapped to a long one before the lesson
 * builds its hand — the centered 5xl string wrapped over two lines on
 * the mobile viewport is the overlap state the owner's UI acceptance
 * canon requires.
 */
export const lessonAcquaintanceWordScenario: ShotScenario = {
  ...descriptors["lesson-acquaintance-word"],
  async run({ page }, sink) {
    await swapSeededWordToLong(page);

    await page.goto("/lesson");
    await page
      .getByTestId("lesson-page")
      .waitFor({ state: "visible", timeout: 90_000 });

    const hand = page.getByTestId("acquaintance-view");
    await hand.waitFor({ state: "visible", timeout: 180_000 });

    // Presentation: walk to the word slide (the long swapped word is
    // the first seeded card) and capture the corner button in the card.
    const next = page.getByTestId("acquaintance-next-btn");
    let presentationShot = false;
    for (let index = 0; index < 12 && !presentationShot; index++) {
      const wordSlide = await page
        .getByTestId("acquaintance-word-slide")
        .isVisible({ timeout: 1_000 })
        .catch(() => false);
      if (wordSlide) {
        await page.waitForTimeout(700);
        await sink.shot("word-presentation");
        presentationShot = true;
        break;
      }
      await next.click({ timeout: 2_000 }).catch(() => undefined);
      await page.waitForTimeout(500);
    }
    expect(
      presentationShot,
      "the hand must show the word presentation slide",
    ).toBe(true);

    // Presentation → training.
    for (let index = 0; index < 12; index++) {
      const training = await page
        .getByTestId("acquaintance-training")
        .isVisible({ timeout: 1_000 })
        .catch(() => false);
      if (training) break;
      await next.click({ timeout: 2_000 }).catch(() => undefined);
      await page.waitForTimeout(500);
    }
    await expect(page.getByTestId("acquaintance-training")).toBeVisible({
      timeout: 30_000,
    });

    // Training: the audio-front coin is the low bit of a fresh Ulid
    // (training_view.rs) — non-deterministic per card entry, so the walk
    // rotates the hand until the central replay button shows up. The
    // text front is captured on the first voiced front without the
    // replay button (the corner button renders exactly there); the
    // audio-front capture proves the corner button is absent on that
    // front — no doubles.
    const front = page.getByTestId("acquaintance-training-front");
    const audioFrontPlay = page.getByTestId("acquaintance-audio-front-play");
    const cornerButton = page.getByTestId("acquaintance-audio-btn");
    const reveal = page.getByTestId("acquaintance-reveal-btn");
    let textFrontShot = false;
    let audioFrontShot = false;
    let answerShot = false;
    for (
      let index = 0;
      index < 30 && !(textFrontShot && audioFrontShot && answerShot);
      index++
    ) {
      if (
        !audioFrontShot &&
        (await audioFrontPlay.isVisible().catch(() => false))
      ) {
        // The audio front: the central replay button owns the card —
        // the corner button must NOT be on screen.
        await expect(cornerButton, "no corner button on the audio front")
          .toHaveCount(0);
        await page.waitForTimeout(500);
        await sink.shot("word-training-audio-front");
        audioFrontShot = true;
      } else if (
        !textFrontShot &&
        (await front.isVisible().catch(() => false)) &&
        !(await audioFrontPlay.isVisible().catch(() => false))
      ) {
        // The text front (word or grammar — the first rotation entry is
        // the long word, it is the presentation's first card). A raw
        // selector inside the marked front: the word text has no
        // marker of its own (a bare serif paragraph).
        await expect(
          front.locator("p.font-serif").first(),
          "the text front shows a serif word",
        ).toBeVisible();
        await expect(
          cornerButton,
          "the corner button sits in the card on the text front",
        ).toBeVisible();
        await page.waitForTimeout(500);
        await sink.shot("word-training-front");
        textFrontShot = true;
      }

      if (!answerShot) {
        await reveal.click({ timeout: 2_000 }).catch(() => undefined);
        const answerVisible = await page
          .getByTestId("acquaintance-training-answer")
          .waitFor({ state: "visible", timeout: 3_000 })
          .then(() => true)
          .catch(() => false);
        if (answerVisible) {
          await expect(
            cornerButton,
            "the corner button returns on the answer side",
          ).toBeVisible();
          await page.waitForTimeout(500);
          await sink.shot("word-training-answer");
          answerShot = true;
          await answerTrainingRemember(page);
        }
      } else {
        // Both front states may still be pending: answer the card to
        // rotate the hand and roll a new audio-front coin.
        await answerTrainingRemember(page);
      }
    }
    expect(textFrontShot, "the text front with the corner button").toBe(true);
    expect(
      audioFrontShot,
      "the audio front without the corner button (bounded walk lost the coin)",
    ).toBe(true);
    expect(answerShot, "the answer side with the corner button").toBe(true);
  },
};

/** The seeded word card the long-word swap rewrites (see the preset). */
const SEEDED_LONG_WORD_CARD = "01JCAX86C980JDZAC4AHN7CP1Y";
const LONG_WORD = "がんばってください";

/**
 * Swap the first seeded word to a long one, in the same IndexedDB record
 * the acquaintance-hand preset seeds — before the lesson builds its hand.
 */
async function swapSeededWordToLong(page: Page): Promise<void> {
  const stored = await readStoredUser(page);
  const user = JSON.parse(stored.value) as {
    knowledge_set?: {
      study_cards?: Record<
        string,
        { card?: Record<string, { word?: { text: string } }> }
      >;
    };
  };
  const vocabulary = user.knowledge_set?.study_cards?.[SEEDED_LONG_WORD_CARD]
    ?.card?.["Vocabulary"];
  if (!vocabulary?.word) {
    throw new Error(
      `seeded word card ${SEEDED_LONG_WORD_CARD} is missing — the acquaintance-hand preset changed its ids`,
    );
  }
  vocabulary.word.text = LONG_WORD;
  await writeStoredUser(page, stored.key, JSON.stringify(user));
}
