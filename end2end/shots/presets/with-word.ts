import { expect } from "@playwright/test";

import { WordsPage } from "../../pages/words.page.ts";
import { fresh } from "./fresh.ts";
import type { PresetContext } from "./index.ts";

/** The word of the demo profile. */
const DEMO_WORD = "私は本を読みます";

/**
 * Add one word through the UI so the app writes a valid Vocabulary card.
 *
 * The page object asserts the selection and the drawer close; the grid
 * count then proves the card was persisted before a preset reads the store.
 */
export async function addWord(
  context: PresetContext,
  text: string,
): Promise<void> {
  const words = new WordsPage(context.page);
  await context.page.goto("/words");
  await words.wordsPage.waitFor({ timeout: 30_000 });
  const before = await words.wordsGrid.getByTestId("words-card-item").count();

  await words.openAddModal();
  await words.enterText(text);
  await words.analyzeText();
  await words.expectSingleAnalyzedWord();
  await words.selectFirstWord();
  await words.addSelectedWords();
  await expect(words.wordsGrid.getByTestId("words-card-item")).toHaveCount(
    before + 1,
    {
      timeout: 20_000,
    },
  );
}

/**
 * A profile with exactly one word.
 *
 * The hand of the lesson is built from the due cards, and a full N4
 * import starts with the grammar and kanji cards — the word slide with
 * the example would be thousands of clicks away. One word makes the hand
 * a single slide.
 */
export async function withWord(context: PresetContext): Promise<void> {
  await fresh(context);
  await addWord(context, DEMO_WORD);
}
