import { readStoredUser, writeStoredUser } from "../seed.ts";
import { onboarded } from "./onboarded.ts";
import type { PresetContext } from "./index.ts";

const day = 24 * 60 * 60 * 1000;
const DUE_LIMIT = 24;

const iso = (ms: number) =>
  new Date(ms).toISOString().replace(/\.\d{3}Z$/, "Z");

/** Words that anchor seeded phrases in the #1753 reachability probes. */
const ANCHOR_WORDS = ["今年", "夏", "分かる"];

type MemoryState = {
  next_review_date?: string;
  stability?: { value?: number };
  difficulty?: { value?: number };
};

interface SeedCard {
  card?: { Vocabulary?: { word?: { text?: string } } } & Record<
    string,
    unknown
  >;
  memory_history?: {
    current_state?: MemoryState | null;
    last_review_date?: string | null;
  } | null;
}

type Cards = Record<string, SeedCard>;

function wordOf(card: SeedCard | undefined): string | null {
  return card?.card?.Vocabulary?.word?.text ?? null;
}

function isVocabulary(card: SeedCard | undefined): boolean {
  return Object.keys(card?.card ?? {})[0] === "Vocabulary";
}

/** Anchor words first, then fill the bounded slice with word cards. */
function pickDueIds(cards: Cards): Set<string> {
  const due = new Set<string>();
  for (const [id, card] of Object.entries(cards)) {
    if (ANCHOR_WORDS.includes(wordOf(card) ?? "")) due.add(id);
  }
  for (const [id, card] of Object.entries(cards)) {
    if (due.size >= DUE_LIMIT) break;
    if (isVocabulary(card)) due.add(id);
  }
  return due;
}

/** Known, low-difficulty memory makes AudioRecall the live audio view. */
function makeKnown(state: MemoryState): void {
  if (state.stability) state.stability.value = 30.0;
  if (state.difficulty) state.difficulty.value = 4.0;
}

/** Returns true when this card was one of the due word cards. */
function patchCard(
  card: SeedCard,
  isDue: boolean,
  dueDate: string,
  pushedDate: string,
  stamp: string,
): boolean {
  const state = card.memory_history?.current_state;
  if (!state) return false;
  state.next_review_date = isDue ? dueDate : pushedDate;
  if (isDue) makeKnown(state);
  if (card.memory_history) card.memory_history.last_review_date = stamp;
  return isDue;
}

/**
 * The onboarded corpus with due vocabulary cards.
 *
 * `due-corpus` keeps only kanji due, but AudioRecall is drawn from
 * review Vocabulary cards. Anchor words go due first so an
 * interleaved phrase can attach to them; every patched card gets a
 * bumped `last_review_date` so the local seed wins the LWW merge.
 */
export async function dueWords(context: PresetContext): Promise<void> {
  await onboarded(context);

  const stored = await readStoredUser(context.page);
  const user = JSON.parse(stored.value) as {
    knowledge_set?: { study_cards?: Cards };
  };
  const cards = user.knowledge_set?.study_cards ?? {};
  const now = Date.now();
  const stamp = iso(now);
  const due = pickDueIds(cards);
  const dueDate = iso(now - day);
  const pushedDate = iso(now + 30 * day);

  let patched = 0;
  for (const [id, card] of Object.entries(cards)) {
    if (patchCard(card, due.has(id), dueDate, pushedDate, stamp)) patched += 1;
  }
  if (patched === 0) {
    throw new Error("due-words: no Vocabulary card with memory state to patch");
  }
  console.log(`[shots] due-words: ${patched} vocabulary cards set due`);
  await writeStoredUser(context.page, stored.key, JSON.stringify(user));
  await context.page.reload({ waitUntil: "domcontentloaded" });
  await context.page.waitForURL(/\/home$/, { timeout: 120_000 });
}
