import { expect, type Page } from "@playwright/test";

import { answerTrainingForgot } from "../../helpers/lesson.ts";
import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { readStoredUser, writeStoredUser } from "../seed.ts";
import { waitForAppReady } from "./simple-page.ts";

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
 * Both seeded words are swapped to long voiced ones before the lesson
 * builds its hand: the centered 5xl string wrapped over several lines on
 * the mobile viewport is the overlap state the owner's UI acceptance
 * canon requires, and the pitch audio keeps the audio-front coin
 * reachable. The walk rates «не помню» — the criterion stays open, so
 * the cards keep cycling and every word entry rolls a fresh audio-front
 * coin (the low bit of a fresh Ulid, training_view.rs) until the central
 * replay button shows up.
 */
export const lessonAcquaintanceWordScenario: ShotScenario = {
  ...descriptors["lesson-acquaintance-word"],
  async run({ page }, sink) {
    await swapSeededWordsToLong(page);

    await page.goto("/lesson");
    await page
      .getByTestId("lesson-page")
      .waitFor({ state: "visible", timeout: 90_000 });
    // The audio-front coin needs the pitch index and the loaded audio
    // before the first training render. Without this wait the first
    // cycle is always a text front, even though the coin is fair.
    await waitForAppReady(page);

    const hand = page.getByTestId("acquaintance-view");
    await hand.waitFor({ state: "visible", timeout: 180_000 });

    // Presentation: walk to the word slide (the long swapped word is
    // the first seeded card) and capture the corner button in the card.
    const next = page.getByTestId("acquaintance-next-btn");
    let presentationShot = false;
    for (let index = 0; index < 12 && !presentationShot; index++) {
      const wordSlide = await page
        .getByTestId("acquaintance-word-slide")
        // locator.isVisible() returns immediately (its timeout option is
        // ignored in Playwright 1.60) — the walk loop below is the wait.
        .isVisible()
        .catch(() => false);
      if (wordSlide) {
        await page.waitForTimeout(700);
        await sink.shot("word-presentation", {
          marker: "acquaintance-word-slide",
        });
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
        .isVisible()
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
    // keeps the cards CYCLING with «не помню» ratings (the criterion
    // stays open, no phase completes) and rolls a fresh coin on every
    // word entry until the central replay button shows up. The text
    // front is captured on the first voiced front without the replay
    // button (the corner button renders exactly there); the audio-front
    // capture proves the corner button is absent on that front — no
    // doubles.
    const front = page.getByTestId("acquaintance-training-front");
    const audioFrontPlay = page.getByTestId("acquaintance-audio-front-play");
    const cornerButton = page.getByTestId("acquaintance-audio-btn");
    const reveal = page.getByTestId("acquaintance-reveal-btn");
    const trainingDone = page
      .getByTestId("acquaintance-completed")
      .or(page.getByTestId("lesson-complete-screen"));
    let textFrontShot = false;
    let audioFrontShot = false;
    let answerShot = false;
    for (
      let index = 0;
      index < 60 &&
      !(textFrontShot && audioFrontShot && answerShot) &&
      !(await trainingDone.isVisible().catch(() => false));
      index++
    ) {
      if (
        !audioFrontShot &&
        (await audioFrontPlay.isVisible().catch(() => false))
      ) {
        // The audio front: the central replay button owns the card —
        // the corner button must NOT be on screen.
        await expect(
          cornerButton,
          "no corner button on the audio front",
        ).toHaveCount(0);
        await page.waitForTimeout(500);
        await sink.shot("word-training-audio-front", {
          marker: "acquaintance-audio-front-play",
        });
        audioFrontShot = true;
      } else if (
        !textFrontShot &&
        (await front.isVisible().catch(() => false)) &&
        !(await audioFrontPlay.isVisible().catch(() => false)) &&
        // Only a VOICED front carries the corner button — but a voiced
        // front is not necessarily a WORD front: grammar fronts render
        // span.font-serif (TranslatorText), the translation side is
        // p.font-mono, a kanji draw has no button at all. Gate the shot
        // on the serif WORD paragraph itself, or the assert below races
        // the draw order and fails at random.
        // Raw selector by sanction: the word paragraph carries no
        // data-testid of its own — p.font-serif is the only marker the
        // markup offers (the same selector is asserted at the capture).
        (await cornerButton.isVisible().catch(() => false)) &&
        (await front
          .locator("p.font-serif")
          .first()
          .isVisible()
          .catch(() => false))
      ) {
        // The word text front: the serif paragraph is the word.
        await expect(
          front.locator("p.font-serif").first(),
          "the text front shows a serif word",
        ).toBeVisible();
        await page.waitForTimeout(500);
        await sink.shot("word-training-front", {
          marker: "acquaintance-training-front",
        });
        textFrontShot = true;
      }

      if (!answerShot) {
        // Only a VOICED card owns the corner button on the answer side:
        // a kanji card has no reading, its answer legitimately shows no
        // button (the wasm contract
        // audio_button_hidden_on_kanji_slide). Ask the front for the
        // voice before revealing — otherwise the assert races the
        // 1-in-5 kanji draw and fails at random.
        // locator.isVisible() returns immediately — this is a polling
        // gate inside the bounded walk, not a wait (the walk loop
        // retries and the real asserts below auto-retry).
        const frontVoiced = await cornerButton.isVisible().catch(() => false);
        if (frontVoiced) {
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
            await sink.shot("word-training-answer", {
              marker: "acquaintance-training-answer",
            });
            answerShot = true;
          }
        }
      }
      // «Не помню» keeps every criterion open — the rotation continues
      // and the next word entry rolls a new coin. On the last missing
      // state the card advances the same way.
      await answerTrainingForgot(page);
    }
    expect(textFrontShot, "the text front with the corner button").toBe(true);
    expect(
      audioFrontShot,
      "the audio front without the corner button (bounded walk lost the coin)",
    ).toBe(true);
    expect(answerShot, "the answer side with the corner button").toBe(true);
  },
};

/** The seeded word cards the long-word swap rewrites (see the preset). */
const SEEDED_WORD_CARDS = [
  "01JCAX86C980JDZAC4AHN7CP1Y",
  "01JCA73QAPHK2GXHPFNHK0FTV8",
];
/** Long voiced words (pitch index has both): a 5xl string over the
 * mobile card width wraps — the overlap state under the corner button. */
const LONG_WORDS = ["国際オリンピック委員会", "朝鮮民主主義人民共和国"];

/**
 * Swap both seeded words to long voiced ones, in the same IndexedDB
 * record the acquaintance-hand preset seeds — before the lesson builds
 * its hand.
 */
async function swapSeededWordsToLong(page: Page): Promise<void> {
  const stored = await readStoredUser(page);
  const user = JSON.parse(stored.value) as {
    knowledge_set?: {
      study_cards?: Record<
        string,
        { card?: Record<string, { word?: { text: string } }> }
      >;
    };
  };
  const cards = user.knowledge_set?.study_cards ?? {};
  for (const [index, cardId] of SEEDED_WORD_CARDS.entries()) {
    const vocabulary = cards[cardId]?.card?.["Vocabulary"];
    if (!vocabulary?.word) {
      throw new Error(
        `seeded word card ${cardId} is missing — the acquaintance-hand preset changed its ids`,
      );
    }
    vocabulary.word.text = LONG_WORDS[index];
  }
  await writeStoredUser(page, stored.key, JSON.stringify(user));
}
