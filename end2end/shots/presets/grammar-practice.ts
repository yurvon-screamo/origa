import { readStoredUser, writeStoredUser } from "../seed.ts";
import { fresh } from "./fresh.ts";
import type { PresetContext } from "./index.ts";

const RULE_ID = "01G00000000000000024000000"; // ～ます, format_map Verb
const RULE_CARD_ID = "01JCBZ8QK0AX4W7YH2Z5P9T3GJ";
const WORD_CARD_ID = "01JCBZ8QK0AX4W7YH2Z5P9T3GH";

/** Stability above the known threshold: practice filters vocabulary by
 * `is_known_card || is_in_progress`, a fresh card has no questions. */
function knownMemory() {
  const stamp = new Date().toISOString().replace(/\.\d{3}Z$/, "Z");
  return {
    current_state: {
      stability: { value: 30.0 },
      difficulty: { value: 5.0 },
      next_review_date: "2030-01-01T00:00:00Z",
      card_state: "Review",
    },
    reps: 5,
    lapses: 0,
    easy_count: 0,
    good_count: 5,
    last_review_date: stamp,
    last_rating: "Good",
    consecutive_again: 0,
    ghost: null,
    marked_known_at: null,
  };
}

/**
 * A quiz-capable grammar rule and one known verb.
 *
 * The BDD flow adds both through the UI; that works but costs minutes
 * per viewport and pushes the scenario into the 20-minute test timeout.
 * Seeding the same two cards directly keeps the captured state
 * user-reachable without the round trips.
 */
export async function grammarPractice(context: PresetContext): Promise<void> {
  await fresh(context);

  const stored = await readStoredUser(context.page);
  const user = JSON.parse(stored.value) as {
    knowledge_set?: { study_cards?: Record<string, unknown> };
  };
  user.knowledge_set ??= {};
  user.knowledge_set.study_cards ??= {};
  const cards = user.knowledge_set.study_cards;
  cards[WORD_CARD_ID] = {
    card_id: WORD_CARD_ID,
    card: {
      Vocabulary: {
        word: { text: "読む" },
        reverse_side: { text: "читать" },
        pos: "Verb",
        tokens: null,
      },
    },
    memory_history: knownMemory(),
    is_favorite: false,
    perfect_streak_since_known: 0,
    favorite_changed_at: null,
  };
  cards[RULE_CARD_ID] = {
    card_id: RULE_CARD_ID,
    card: { Grammar: { rule_id: RULE_ID } },
    memory_history: {},
    is_favorite: false,
    perfect_streak_since_known: 0,
    favorite_changed_at: null,
  };

  await writeStoredUser(context.page, stored.key, JSON.stringify(user));
  await context.page.reload({ waitUntil: "domcontentloaded" });
  await context.page.waitForURL(/\/home$/, { timeout: 120_000 });
}
