import { readStoredUser, writeStoredUser } from "../seed.ts";
import { fresh } from "./fresh.ts";
import { addWord } from "./with-word.ts";
import type { PresetContext } from "./index.ts";

/** The prod phrase behind #914 (card 53/65, chunk p0024). */
const PROD_PHRASE_ID = "00000000003YTM62ZWSHWQPHH5";

/** The study-card id of the injected card (a syntactically valid ULID). */
const CARD_ID = "PM4Q528M101GNGSPPPZCY2PHFH";

/** Grammar rules of the phrase in the local index; boot removes the card without them. */
const GRAMMAR_RULE_IDS = [
  "01G0000000000000006W000000",
  "01G000000000000000JC000000",
] as const;

/** Distinct study-card ids for the grammar cards. */
const GRAMMAR_CARD_IDS = [
  "PM4Q528M101GNGSPPPZCY2PHFG",
  "PM4Q528M101GNGSPPPZCY2PHFJ",
] as const;

/** Known tokens of the target phrase; only the anchor enters the lesson core. */
const PHRASE_WORDS = ["少し", "実"] as const;
const ANCHOR_WORD = "少し";

const iso = (date: Date) => date.toISOString().replace(/\.\d{3}Z$/, "Z");

/** A known memory state; due words are past, grammar cards stay out of the core. */
function knownMemory(nextReviewDate: string, stamp: string) {
  return {
    current_state: {
      stability: { value: 30.0 },
      difficulty: { value: 4.0 },
      next_review_date: nextReviewDate,
      card_state: "Review",
    },
    reps: 1,
    lapses: 0,
    easy_count: 0,
    good_count: 1,
    last_review_date: stamp,
    last_rating: "Good",
    consecutive_again: 0,
  };
}

interface SeedCard {
  card_id?: string;
  card?: Record<string, unknown>;
  memory_history?: Record<string, unknown>;
  is_favorite?: boolean;
  perfect_streak_since_known?: number;
  favorite_changed_at?: string | null;
}

interface SeedUser {
  knowledge_set?: { study_cards?: Record<string, SeedCard> };
}

function vocabularyWord(card: SeedCard | undefined): string | undefined {
  const vocabulary = card?.card?.Vocabulary as
    | { word?: { text?: string } }
    | undefined;
  return vocabulary?.word?.text;
}

/**
 * A fresh profile carrying the dirty prod phrase as its only due phrase.
 *
 * The words and grammar rules make the phrase boot-eligible and give the
 * lesson a non-empty core. The chunk bytes stay dirty on the CDN; the
 * client sanitizer owns the cleanup on load.
 */
export async function phraseDashRun(context: PresetContext): Promise<void> {
  await fresh(context);
  for (const word of PHRASE_WORDS) {
    await addWord(context, word);
  }

  const stored = await readStoredUser(context.page);
  const user = JSON.parse(stored.value) as SeedUser;
  const cards = (user.knowledge_set ??= { study_cards: {} }).study_cards ?? {};
  user.knowledge_set.study_cards = cards;

  const now = new Date();
  const stamp = iso(now);
  const past = iso(new Date(now.getTime() - 24 * 60 * 60 * 1000));
  const future = iso(new Date(now.getTime() + 30 * 24 * 60 * 60 * 1000));

  for (const word of PHRASE_WORDS) {
    const card = Object.values(cards).find(
      (entry) => vocabularyWord(entry) === word,
    );
    if (!card)
      throw new Error(`phrase-dash-run: word card ${word} was not created`);
    // The second word feeds the known pool only; a future date keeps it out
    // of the core and leaves one word to walk before the anchored phrase.
    card.memory_history = knownMemory(
      word === ANCHOR_WORD ? past : future,
      stamp,
    );
  }

  // A favorite phrase is pinned into the lesson core first, so the scenario
  // reaches the question front without rating the word cards.
  cards[CARD_ID] = {
    card_id: CARD_ID,
    card: { Phrase: { phrase_id: PROD_PHRASE_ID } },
    memory_history: knownMemory(past, stamp),
    is_favorite: true,
    perfect_streak_since_known: 0,
    favorite_changed_at: stamp,
  };

  GRAMMAR_RULE_IDS.forEach((ruleId, index) => {
    const id = GRAMMAR_CARD_IDS[index];
    cards[id] = {
      card_id: id,
      card: { Grammar: { rule_id: ruleId } },
      memory_history: knownMemory(future, stamp),
      is_favorite: false,
      perfect_streak_since_known: 0,
      favorite_changed_at: null,
    };
  });

  await writeStoredUser(context.page, stored.key, JSON.stringify(user));
  await context.page.reload({ waitUntil: "domcontentloaded" });
  await context.page.goto("/home");
  await context.page
    .getByTestId("home-page")
    .waitFor({ state: "visible", timeout: 120_000 });
}
