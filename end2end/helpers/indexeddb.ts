import { type Page } from "@playwright/test";

/**
 * Reader for the local store of the app: IndexedDB `origa` → `users`.
 *
 * Single source of truth for the steps that must observe a persisted write
 * (the counter seed step writes the same structure, so both sides have to
 * change together).
 *
 * The stored value is the user as a JSON string; the row that carries an
 * `email` field is the current user.
 */

const DATABASE_NAME = "origa";
const USER_STORE = "users";

/** Memory of one counter binding (a digit × suffix pair). */
export interface PersistedCounterBinding {
    number?: number;
    memory?: {
        current_state?: unknown;
    } | null;
}

/** One study card of the persisted knowledge set. */
export interface PersistedStudyCard {
    card?: {
        Counter?: {
            bindings?: PersistedCounterBinding[];
        };
    };
    memory_history?: {
        current_state?: unknown;
        reps?: number;
    };
}

/** The persisted user (only the parts the tests read). */
export interface PersistedUser {
    knowledge_set?: {
        study_cards?: Record<string, PersistedStudyCard>;
    };
}

/** Keys of the local user store (the user id is the key). */
export async function readUserStoreKeys(page: Page): Promise<string[]> {
    return await page.evaluate(
        async ({ databaseName, userStore }): Promise<string[]> => {
            return await new Promise<string[]>((resolve, reject) => {
                const open = indexedDB.open(databaseName);
                open.onerror = () => reject(new Error("indexedDB open failed"));
                open.onupgradeneeded = () => resolve([]);
                open.onsuccess = () => {
                    const db = open.result;
                    if (!db.objectStoreNames.contains(userStore)) {
                        resolve([]);
                        return;
                    }
                    const tx = db.transaction(userStore, "readonly");
                    const store = tx.objectStore(userStore);
                    const req = store.getAllKeys();
                    req.onsuccess = () => resolve(req.result.map((k) => String(k)));
                    req.onerror = () => reject(new Error("getAllKeys failed"));
                };
            });
        },
        { databaseName: DATABASE_NAME, userStore: USER_STORE },
    );
}

/** The persisted user, or `null` when the store holds no user yet. */
export async function readPersistedUser(page: Page): Promise<PersistedUser | null> {
    const raw = await page.evaluate(
        async ({ databaseName, userStore }): Promise<string | null> => {
            return await new Promise<string | null>((resolve, reject) => {
                const open = indexedDB.open(databaseName);
                open.onerror = () => reject(new Error("indexedDB open failed"));
                open.onsuccess = () => {
                    const db = open.result;
                    if (!db.objectStoreNames.contains(userStore)) {
                        resolve(null);
                        return;
                    }
                    const tx = db.transaction(userStore, "readonly");
                    const store = tx.objectStore(userStore);
                    const req = store.getAll();
                    req.onsuccess = () => {
                        const row = req.result.find(
                            (value: unknown) =>
                                typeof value === "string" && value.includes('"email"'),
                        );
                        resolve(typeof row === "string" ? row : null);
                    };
                    req.onerror = () => reject(new Error("getAll failed"));
                };
            });
        },
        { databaseName: DATABASE_NAME, userStore: USER_STORE },
    );

    if (raw === null) {
        return null;
    }
    try {
        return JSON.parse(raw) as PersistedUser;
    } catch {
        return null;
    }
}

/**
 * How many counter bindings already carry a memory state.
 *
 * A binding starts empty (`current_state: null`); the first review of a
 * binding seeds its memory, so the counter grows from 0 to 1 after a
 * rating. The semantic memory of the card itself is NOT a signal here: a
 * binding review does not touch it.
 */
export function countReviewedCounterBindings(user: PersistedUser | null): number {
    const cards = Object.values(user?.knowledge_set?.study_cards ?? {});
    return cards.reduce((total, card) => {
        const bindings = card.card?.Counter?.bindings ?? [];
        return (
            total +
            bindings.filter((binding) => binding.memory?.current_state != null).length
        );
    }, 0);
}
