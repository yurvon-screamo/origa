import type { Page } from "@playwright/test";

/**
 * The local user record of the app (IndexedDB, store `users`).
 *
 * The screenshots of a seeded state need to write into that record — the
 * app reads its corpus from there, not from the server.
 */
export interface StoredUser {
    key: string;
    value: string;
}

export async function readStoredUser(page: Page): Promise<StoredUser> {
    return await page.evaluate(async () => {
        const db = await new Promise<IDBDatabase>((resolve, reject) => {
            const request = indexedDB.open("origa");
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });
        const tx = db.transaction("users", "readonly");
        const store = tx.objectStore("users");
        const keys = (await new Promise<IDBValidKey[]>((resolve, reject) => {
            const request = store.getAllKeys();
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        })) as IDBValidKey[];
        for (const key of keys) {
            if (typeof key !== "string" || !key.startsWith("user:")) continue;
            const value = (await new Promise<unknown>((resolve, reject) => {
                const request = store.get(key);
                request.onsuccess = () => resolve(request.result);
                request.onerror = () => reject(request.error);
            })) as unknown;
            if (typeof value === "string") {
                db.close();
                return { key, value };
            }
        }
        db.close();
        throw new Error("no JSON-string user record found in IndexedDB");
    });
}

export async function writeStoredUser(page: Page, key: string, value: string): Promise<void> {
    await page.evaluate(
        ({ key, value }) =>
            new Promise<void>((resolve, reject) => {
                const open = indexedDB.open("origa");
                open.onerror = () => reject(open.error);
                open.onsuccess = () => {
                    const db = open.result;
                    const tx = db.transaction("users", "readwrite");
                    tx.objectStore("users").put(value, key);
                    tx.oncomplete = () => {
                        db.close();
                        resolve();
                    };
                    tx.onerror = () => reject(tx.error);
                };
            }),
        { key, value },
    );
}

/** RFC3339 without fractional seconds — the format the domain writes. */
function rfc3339(date: Date): string {
    return date.toISOString().replace(/\.\d{3}Z$/, "Z");
}

function daysAgoFromNow(days: number, hourUtc = 12): string {
    const date = new Date();
    date.setUTCDate(date.getUTCDate() - days);
    date.setUTCHours(hourUtc, 0, 0, 0);
    return rfc3339(date);
}

function daysFromNow(days: number, hourUtc = 12): string {
    const date = new Date();
    date.setUTCDate(date.getUTCDate() + days);
    date.setUTCHours(hourUtc, 0, 0, 0);
    return rfc3339(date);
}

/**
 * Seeds a typical active-user state into the stored user JSON: daily
 * ratings spread across the month (today intentionally absent — a partial
 * day would draw a misleading last point), and a couple of troubled words
 * (lapses) for the difficult counter.
 *
 * All values follow the domain serialization the app itself writes.
 */
export function seedTypicalUserState(userJson: string): string {
    const user = JSON.parse(userJson);
    const knowledgeSet = user.knowledge_set;

    // Daily history: ratings across the month, with a few runs of
    // consecutive days so the accuracy chart draws connected segments
    // (isolated points render as dots without a line). Today is
    // intentionally absent — a partial day reads as a dip on the chart.
    // StatsTracker is #[serde(flatten)] inside
    // KnowledgeSet, so lesson_history sits at the TOP level of the
    // knowledge_set JSON — there is no "stats" key on the wire.
    const history: Array<Record<string, unknown>> = Array.isArray(knowledgeSet.lesson_history)
        ? knowledgeSet.lesson_history
        : [];
    for (const days of [2, 4, 6, 8, 10, 12, 14, 15, 16, 18, 20, 21, 22, 24, 26, 27, 28]) {
        const positive = 8 + ((days * 3) % 4);
        const negative = 2 + (days % 3);
        history.push({
            timestamp: daysAgoFromNow(days),
            avg_stability: 12.4,
            avg_difficulty: 4.6,
            total_words: 320,
            new_words: 40,
            known_words: 190,
            in_progress_words: 78,
            high_difficulty_words: 12,
            new_cards_studied_today: 5,
            phrase_cards_studied_today: 2,
            lessons_completed: 2,
            positive_ratings: positive,
            negative_ratings: negative,
            total_ratings: positive + negative,
        });
    }
    knowledgeSet.lesson_history = history;

    // Troubled words: a realistic slice of the corpus (roughly one in
    // twenty-five) carries lapses so the share reads as "normal wear".
    const cards = knowledgeSet.study_cards ?? {};
    const entries = Object.entries(cards);
    const troubledTarget = Math.max(3, Math.floor(entries.length / 25));
    let troubled = 0;
    for (const [, card] of entries) {
        const memory = (card as { memory_history?: Record<string, unknown> }).memory_history;
        if (!memory || !memory.current_state) continue;
        memory.lapses = troubled % 4 === 0 ? 4 : 3;
        memory.consecutive_again = 0;
        troubled += 1;
        if (troubled >= troubledTarget) break;
    }

    return JSON.stringify(user);
}


/** Phrases and examples are not counted by the due counter (issue #809). */
function isCountedType(card: Record<string, unknown>): boolean {
    const keys = Object.keys(card ?? {});
    return !keys.includes("Phrase") && !keys.includes("Example");
}

interface StudyCard {
    card?: Record<string, unknown>;
    memory_history?: {
        current_state?: { next_review_date?: string } | null;
        last_review_date?: string | null;
    };
}

interface UserJson {
    knowledge_set: {
        study_cards?: Record<string, StudyCard>;
    };
}

/** Cards whose review date has come — the number the due counter shows. */
export function countDue(user: UserJson, now: Date): number {
    const cards = Object.values(user.knowledge_set?.study_cards ?? {});
    const nowMs = now.getTime();
    return cards.filter((card) => {
        const state = card.memory_history?.current_state;
        if (!state?.next_review_date) return false;
        if (!isCountedType(card.card ?? {})) return false;
        return new Date(state.next_review_date).getTime() <= nowMs;
    }).length;
}

/**
 * Rewrites the review dates: every card gets a future date, exactly
 * `dueTarget` countable cards get a past one. `last_review_date` is bumped
 * to "now" on every patched card so the local seed wins the LWW merge
 * against the server snapshot.
 */
export function seedDueCards(userJson: string, now: Date, dueTarget: number): { json: string; due: number } {
    const user = JSON.parse(userJson) as UserJson;
    const cards = Object.values(user.knowledge_set?.study_cards ?? {});

    const future = daysFromNow(30);
    const past = daysFromNow(-1);
    const nowStamp = rfc3339(now);

    let due = 0;
    for (const card of cards) {
        const memory = card.memory_history;
        // A card without a review state is new — the seed must not invent
        // one, that would move the card out of the "new" bucket.
        if (!memory?.current_state) continue;

        const countable = isCountedType(card.card ?? {});
        if (countable && due < dueTarget) {
            memory.current_state.next_review_date = past;
            due += 1;
        } else {
            memory.current_state.next_review_date = future;
        }
        memory.last_review_date = nowStamp;
    }

    return { json: JSON.stringify(user), due };
}
