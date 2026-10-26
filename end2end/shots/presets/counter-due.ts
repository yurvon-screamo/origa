import { skipOnboarding } from "../../helpers/navigation.ts";
import { isOnboardingShown } from "../../helpers/onboarding.ts";
import { readStoredUser, writeStoredUser } from "../seed.ts";
import type { PresetContext } from "./index.ts";

const day = 24 * 60 * 60 * 1000;

const iso = (ms: number) => new Date(ms).toISOString().replace(/\.\d{3}Z$/, "Z");

/** Valid ULID for the injected card. */
const COUNTER_CARD_ID = "01JCBZ8QK0AX4W7YH2Z5P9T3GE";

/** The bindings the pack will ask (numbers of the 本 registry). */
const BINDING_NUMBERS = [1, 2, 3, 0];

/**
 * The wire shape of a due review memory (the same contract the BDD
 * counter fixture injects — see end2end/bdd/steps/counter.steps.ts —
 * and the seeded ledger/example cards use; the fields beyond the read
 * interface are serde defaults the app fills in).
 */
interface DueMemory {
    current_state: {
        stability: { value: number };
        difficulty: { value: number };
        next_review_date: string;
        card_state: string;
    };
    reps: number;
    lapses: number;
    easy_count: number;
    good_count: number;
    last_review_date: string;
    last_rating: string;
    consecutive_again: number;
    ghost: null;
    marked_known_at: null;
}

interface SeedCard {
    card_id?: string;
    card?: Record<string, unknown>;
    memory_history?: DueMemory | null;
    is_favorite?: boolean;
    perfect_streak_since_known?: number;
    favorite_changed_at?: string | null;
}

/** A due review memory (yesterday's review date). */
function dueMemory(now: number, stamp: string): DueMemory {
    return {
        current_state: {
            stability: { value: 3.0 },
            difficulty: { value: 5.0 },
            next_review_date: iso(now - day),
            card_state: "Review",
        },
        reps: 1,
        lapses: 0,
        easy_count: 0,
        good_count: 1,
        last_review_date: stamp,
        last_rating: "Good",
        consecutive_again: 0,
        ghost: null,
        marked_known_at: null,
    };
}

/**
 * An otherwise empty profile with a single due COUNTER card (本).
 *
 * The counter card is self-contained (the readings resolve from the
 * in-app registry), so no corpus import is needed: the onboarding is
 * skipped and the card with four due bindings is seeded straight into
 * the local user record. The next lesson opens with the counter
 * bindings pack — the shared counter slot of «number × suffix»
 * showings (issue #415).
 */
export async function counterDue(context: PresetContext): Promise<void> {
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
    // Push every pre-existing card out of the way — the pack must be
    // the first thing the lesson shows.
    for (const card of Object.values(cards)) {
        const state = card?.memory_history?.current_state;
        if (!state) continue;
        state.next_review_date = iso(now + 30 * day);
        if (card.memory_history) card.memory_history.last_review_date = stamp;
    }

    const memory = dueMemory(now, stamp);
    cards[COUNTER_CARD_ID] = {
        card_id: COUNTER_CARD_ID,
        card: {
            Counter: {
                suffix: "本",
                bindings: BINDING_NUMBERS.map((number) => ({ number, memory })),
            },
        },
        memory_history: memory,
        is_favorite: false,
        perfect_streak_since_known: 0,
        favorite_changed_at: null,
    };

    await writeStoredUser(context.page, stored.key, JSON.stringify(user));
    await context.page.reload({ waitUntil: "domcontentloaded" });
    await context.page.waitForURL(/\/home$/, { timeout: 120_000 });
}
