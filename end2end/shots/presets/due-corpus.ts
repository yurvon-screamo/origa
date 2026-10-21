import { readStoredUser, writeStoredUser } from "../seed.ts";
import { onboarded } from "./onboarded.ts";
import type { PresetContext } from "./index.ts";

const day = 24 * 60 * 60 * 1000;

const iso = (ms: number) => new Date(ms).toISOString().replace(/\.\d{3}Z$/, "Z");

function cardKind(card: Record<string, unknown> | undefined): string {
    return Object.keys(card ?? {})[0] ?? "";
}

interface SeedCard {
    card?: Record<string, unknown>;
    memory_history?: {
        current_state?: { next_review_date?: string } | null;
        last_review_date?: string | null;
    } | null;
}

/**
 * The onboarded corpus with a curated due queue.
 *
 * `onboarded` marks every imported card known, which schedules them weeks
 * out — the review body of such a profile renders only Normal showings
 * (the view generator runs for DUE cards, and the thin core is padded
 * with Normal new cards). This preset rewrites the review dates: kanji
 * and example cards become due today, everything else is pushed a month
 * out. The next lesson's core is then built almost entirely of kanji —
 * whose review view is a random mix of reading quiz / meaning quiz /
 * yesno / writing / normal — plus the due example cards mixed in last.
 *
 * Every patched card carries a bumped `last_review_date` so the local
 * seed wins the LWW merge against the server snapshot (same contract as
 * `seedDueCards`).
 */
export async function dueCorpus(context: PresetContext): Promise<void> {
    await onboarded(context);

    const stored = await readStoredUser(context.page);
    const user = JSON.parse(stored.value) as {
        knowledge_set?: { study_cards?: Record<string, SeedCard> };
    };
    const cards = user.knowledge_set?.study_cards ?? {};
    const now = Date.now();
    const stamp = iso(now);
    for (const card of Object.values(cards)) {
        const state = card?.memory_history?.current_state;
        if (!state) continue;
        const kind = cardKind(card.card);
        if (kind === "Kanji") {
            state.next_review_date = iso(now - day);
        } else {
            state.next_review_date = iso(now + 30 * day);
        }
        if (card.memory_history) card.memory_history.last_review_date = stamp;
    }
    await writeStoredUser(context.page, stored.key, JSON.stringify(user));
    await context.page.reload({ waitUntil: "domcontentloaded" });
    await context.page.waitForURL(/\/home$/, { timeout: 120_000 });
}
