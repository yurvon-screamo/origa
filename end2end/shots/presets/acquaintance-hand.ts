import { skipOnboarding } from "../../helpers/navigation.ts";
import { isOnboardingShown } from "../../helpers/onboarding.ts";
import { readStoredUser, writeStoredUser } from "../seed.ts";
import type { PresetContext } from "./index.ts";

/**
 * A minimal hand-picked corpus: two words, one kanji, two grammar rules.
 *
 * The acquaintance hand is selected from the new cards with type weights,
 * so a real import leaves the grammar slide's presence to chance (~10%
 * share per draw). Seeding the whole corpus makes the hand deterministic:
 * the weights renormalize over the available types and every seeded card
 * lands in the hand of the next lesson. No onboarding import — the state
 * builds in seconds, not minutes.
 *
 * The grammar rule ids come from the CDN grammar registry
 * (`grammar/grammar_v3.json`, N5 section) — the lesson resolves the slide
 * content from the registry by the card's rule_id.
 */
const SEEDED_CARDS: Array<{ id: string; card: Record<string, unknown> }> = [
    {
        id: "01JCAX86C980JDZAC4AHN7CP1Y",
        card: {
            Vocabulary: {
                word: { text: "先生" },
                reverse_side: { text: "учитель" },
                pos: null,
                tokens: null,
            },
        },
    },
    {
        id: "01JCA73QAPHK2GXHPFNHK0FTV8",
        card: {
            Vocabulary: {
                word: { text: "学生" },
                reverse_side: { text: "студент" },
                pos: null,
                tokens: null,
            },
        },
    },
    {
        id: "01JCAV0KQVYD4F5H8P0EQ5Z792",
        card: { Kanji: { kanji: { text: "日" } } },
    },
    {
        id: "01JCAHGP8PDPMSG05VZE3S45EN",
        card: { Grammar: { rule_id: "01KJ9AVWBGC2BT0DMFPDYYFEWB" } },
    },
    {
        id: "01JCAJMJK83RHY1KNMBDTKB7SY",
        card: { Grammar: { rule_id: "01G00000000000000004000000" } },
    },
];

export async function acquaintanceHand(context: PresetContext): Promise<void> {
    if (await isOnboardingShown(context.page)) {
        // A plain skip: no import — the corpus is seeded right below.
        await skipOnboarding(context.page);
    }

    const stored = await readStoredUser(context.page);
    const user = JSON.parse(stored.value) as {
        knowledge_set?: { study_cards?: Record<string, unknown> };
    };
    user.knowledge_set ??= {};
    user.knowledge_set.study_cards = Object.fromEntries(
        SEEDED_CARDS.map((entry) => [
            entry.id,
            {
                card_id: entry.id,
                card: entry.card,
                // All-default MemoryHistory: `current_state: null` — the
                // card is new (the seed must not invent a review state).
                memory_history: {},
                is_favorite: false,
                perfect_streak_since_known: 0,
                favorite_changed_at: null,
            },
        ]),
    );
    await writeStoredUser(context.page, stored.key, JSON.stringify(user));

    // The reload re-reads the local record; the mount sync merges the
    // (empty) server snapshot with it — the seeded cards survive.
    await context.page.reload({ waitUntil: "domcontentloaded" });
    await context.page.waitForURL(/\/home$/, { timeout: 120_000 });
}
