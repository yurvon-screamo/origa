import { readStoredUser, writeStoredUser } from "../seed.ts";
import { fresh } from "./fresh.ts";
import { withWord } from "./with-word.ts";
import type { PresetContext } from "./index.ts";

/** The prod phrase behind #914 (card 53/65, chunk p0024). */
const PROD_PHRASE_ID = "00000000003YTM62ZWSHWQPHH5";

/** The study-card id of the injected card (a syntactically valid ULID). */
const CARD_ID = "PM4Q528M101GNGSPPPZCY2PHFH";

/**
 * The persisted study card: the prod phrase, due for review.
 *
 * The shape mirrors the persisted schema the counter seed step and
 * due-corpus write (db=origa, store=users, nested JSON string) — a
 * schema drift there needs a synchronous edit here.
 */
const DUE_PHRASE_CARD = {
  card_id: CARD_ID,
  card: { Phrase: { phrase_id: PROD_PHRASE_ID } },
  memory_history: {
    current_state: {
      stability: { value: 3.0 },
      difficulty: { value: 5.0 },
      next_review_date: "2020-01-01T00:00:00Z",
      card_state: "Review",
    },
    reps: 1,
    lapses: 0,
    easy_count: 0,
    good_count: 1,
    last_review_date: "2020-01-01T00:00:00Z",
    last_rating: "Good",
    consecutive_again: 0,
  },
};

/**
 * A fresh profile carrying the dirty prod phrase as its only due card
 * (#914).
 *
 * The phrases list shows the card, and the lesson review core is built
 * from it. A profile that never adds cards may not have persisted a
 * user row yet — the words flow creates it, so the bootstrap adds one
 * word through the UI when the store is still empty (the extra new
 * word card only pads the acquaintance hand ahead of the phrase).
 */
export async function phraseDashRun(context: PresetContext): Promise<void> {
  await fresh(context);

  let stored;
  try {
    stored = await readStoredUser(context.page);
  } catch {
    await withWord(context);
    stored = await readStoredUser(context.page);
  }

  const user = JSON.parse(stored.value) as {
    knowledge_set?: { study_cards?: Record<string, unknown> };
  };
  user.knowledge_set = user.knowledge_set ?? {};
  user.knowledge_set.study_cards = user.knowledge_set.study_cards ?? {};
  user.knowledge_set.study_cards[CARD_ID] = DUE_PHRASE_CARD;
  await writeStoredUser(context.page, stored.key, JSON.stringify(user));

  await context.page.reload({ waitUntil: "domcontentloaded" });
  await context.page.waitForURL(/\/home$/, { timeout: 120_000 });
}
