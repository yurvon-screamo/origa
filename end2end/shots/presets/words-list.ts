import { skipOnboarding } from "../../helpers/navigation.ts";
import { isOnboardingShown } from "../../helpers/onboarding.ts";
import { readStoredUser, writeStoredUser } from "../seed.ts";
import type { PresetContext } from "./index.ts";

/**
 * A handful of vocabulary cards the words screen can list without a
 * corpus import: the header (title + tab strip + add button) is the
 * surface the words captures exist for, the list only needs plausible
 * rows. The cards carry no memory state, so they render in the "new"
 * bucket — the look of a profile that just started adding words.
 */
const WORDS = ["ねこ", "辞書", "食べる", "難しい"];

export async function wordsList(context: PresetContext): Promise<void> {
  if (await isOnboardingShown(context.page)) {
    await skipOnboarding(context.page);
  }

  const stored = await readStoredUser(context.page);
  const user = JSON.parse(stored.value) as {
    knowledge_set?: { study_cards?: Record<string, unknown> };
  };
  user.knowledge_set ??= {};
  const cards = (user.knowledge_set.study_cards ??= {});

  WORDS.forEach((word, i) => {
    // Valid ULIDs: the shared counter-due prefix with a varying tail.
    const id = `01JCBZ8QK0AX4W7YH2Z5P9T3G${i}`;
    cards[id] = {
      card_id: id,
      card: {
        Vocabulary: { word: { text: word }, reverse_side: null, pos: null },
      },
      memory_history: null,
      is_favorite: false,
      perfect_streak_since_known: 0,
      favorite_changed_at: null,
    };
  });

  await writeStoredUser(context.page, stored.key, JSON.stringify(user));
  await context.page.reload({ waitUntil: "domcontentloaded" });
  await context.page.waitForURL(/\/home$/, { timeout: 120_000 });
}
