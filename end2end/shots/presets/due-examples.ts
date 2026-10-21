import { skipOnboarding } from "../../helpers/navigation.ts";
import { isOnboardingShown } from "../../helpers/onboarding.ts";
import { readStoredUser, writeStoredUser } from "../seed.ts";
import type { PresetContext } from "./index.ts";

const day = 24 * 60 * 60 * 1000;

const iso = (ms: number) => new Date(ms).toISOString().replace(/\.\d{3}Z$/, "Z");

/** Valid ULIDs for the injected cards. */
const EXAMPLE_CARDS: Array<{ id: string; word: string; sentenceId: number }> = [
    { id: "01JCAX86C980JDZAC4AHN7CP2Z", word: "彼", sentenceId: 0 },
    { id: "01JCA73QAPHK2GXHPFNHK0FTW9", word: "家族", sentenceId: 1 },
    { id: "01JCAV0KQVYD4F5H8P0EQ5Z800", word: "生活", sentenceId: 2 },
];

interface SeedCard {
    card?: Record<string, unknown>;
    memory_history?: {
        current_state?: Record<string, unknown> | null;
        last_review_date?: string | null;
    } | null;
}

/**
 * An otherwise empty profile with due EXAMPLE cards only.
 *
 * The example card is self-contained (the sentence content loads from
 * the CDN by sentence_id), so no corpus import is needed: the onboarding
 * is skipped and the due example SRS cards are seeded straight into the
 * local user record. The next lesson consists of example recall cards
 * only: sentence question → reveal → token translation.
 */
export async function dueExamples(context: PresetContext): Promise<void> {
    if (await isOnboardingShown(context.page)) {
        await skipOnboarding(context.page);
    }

    const stored = await readStoredUser(context.page);
    const user = JSON.parse(stored.value) as {
        knowledge_set?: { study_cards?: Record<string, SeedCard> };
    };
    user.knowledge_set ??= {};
    const cards = (user.knowledge_set.study_cards ??= {});

    const now = Date.now();
    const stamp = iso(now);
    for (const card of Object.values(cards)) {
        const state = card?.memory_history?.current_state;
        if (!state) continue;
        if (typeof state === "object") {
            state.next_review_date = iso(now + 30 * day);
        }
        if (card.memory_history) card.memory_history.last_review_date = stamp;
    }

    for (const example of EXAMPLE_CARDS) {
        // The wire shape mirrors the seeded ledger cards: the memory
        // history fields beyond the read interface are defaults the app
        // fills from serde defaults.
        cards[example.id] = {
            card_id: example.id,
            card: { Example: { word: example.word, sentence_id: example.sentenceId } },
            memory_history: {
                current_state: {
                    stability: { value: 6.0 },
                    difficulty: { value: 5.0 },
                    next_review_date: iso(now - day),
                    card_state: "Review",
                },
                reps: 3,
                lapses: 0,
                easy_count: 0,
                good_count: 3,
                last_review_date: stamp,
                last_rating: "Good",
                consecutive_again: 0,
                ghost: null,
                marked_known_at: null,
            },
            is_favorite: false,
            perfect_streak_since_known: 0,
            favorite_changed_at: null,
        } as SeedCard;
    }

    await writeStoredUser(context.page, stored.key, JSON.stringify(user));
    await context.page.reload({ waitUntil: "domcontentloaded" });
    await context.page.waitForURL(/\/home$/, { timeout: 120_000 });
}
