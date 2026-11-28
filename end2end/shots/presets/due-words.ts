import { readStoredUser, writeStoredUser } from "../seed.ts";
import { onboarded } from "./onboarded.ts";
import type { PresetContext } from "./index.ts";

const day = 24 * 60 * 60 * 1000;

const iso = (ms: number) =>
  new Date(ms).toISOString().replace(/\.\d{3}Z$/, "Z");

/** Words that anchor the two seeded phrases of the phrase-listen preset. */
const ANCHOR_WORDS = ["今年", "夏", "分かる"];

interface SeedCard {
  card?: {
    Vocabulary?: { word?: { text?: string } };
  } & Record<string, unknown>;
  memory_history?: {
    current_state?: { next_review_date?: string } | null;
    last_review_date?: string | null;
  } | null;
}

function wordOf(card: SeedCard | undefined): string | null {
  return card?.card?.Vocabulary?.word?.text ?? null;
}

/**
 * The onboarded corpus with due vocabulary cards.
 *
 * `due-corpus` keeps only kanji due, but the AudioRecall view is drawn
 * from review Vocabulary cards. Anchor words go due first so an
 * interleaved phrase can attach to them; the rest of the slice is
 * bounded. Every patched card gets a bumped `last_review_date` so the
 * local seed wins the LWW merge against the server snapshot.
 */
export async function dueWords(context: PresetContext): Promise<void> {
  await onboarded(context);

  const stored = await readStoredUser(context.page);
  const user = JSON.parse(stored.value) as {
    knowledge_set?: { study_cards?: Record<string, SeedCard> };
  };
  const cards = user.knowledge_set?.study_cards ?? {};
  const now = Date.now();
  const stamp = iso(now);
  const due = new Set<string>();

  for (const [id, card] of Object.entries(cards)) {
    if (ANCHOR_WORDS.includes(wordOf(card) ?? "")) due.add(id);
  }
  for (const [id, card] of Object.entries(cards)) {
    if (due.size >= 24) break;
    if (Object.keys(card.card ?? {})[0] === "Vocabulary") due.add(id);
  }

  let patched = 0;
  for (const [id, card] of Object.entries(cards)) {
    const state = card?.memory_history?.current_state;
    if (!state) continue;
    if (due.has(id)) {
      state.next_review_date = iso(now - day);
      // A known, low-difficulty state makes the AudioRecall branch the
      // only audio-driven view available for due vocabulary cards.
      const current = card.memory_history?.current_state as
        | { stability?: { value?: number }; difficulty?: { value?: number } }
        | undefined;
      if (current) {
        if (current.stability) current.stability.value = 30.0;
        if (current.difficulty) current.difficulty.value = 4.0;
      }
      patched += 1;
    } else {
      state.next_review_date = iso(now + 30 * day);
    }
    if (card.memory_history) card.memory_history.last_review_date = stamp;
  }
  if (patched === 0) {
    throw new Error("due-words: no Vocabulary card with memory state to patch");
  }
  console.log(`[shots] due-words: ${patched} vocabulary cards set due`);
  await writeStoredUser(context.page, stored.key, JSON.stringify(user));
  await context.page.reload({ waitUntil: "domcontentloaded" });
  await context.page.waitForURL(/\/home$/, { timeout: 120_000 });
}
