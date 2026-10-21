import { readStoredUser, writeStoredUser } from "../seed.ts";
import type { PresetContext } from "./index.ts";

/**
 * Writes a seeded state into the local user record and reloads.
 *
 * The app reads its corpus from IndexedDB, not from the server, so a
 * seeded screen needs the record rewritten and the app reloaded. The
 * reload also re-runs the mount sync, which merges the server snapshot
 * with the local one — the seeded parts survive that merge.
 */
export async function withSeededUser(
    context: PresetContext,
    seed: (userJson: string) => string,
): Promise<void> {
    const stored = await readStoredUser(context.page);
    await writeStoredUser(context.page, stored.key, seed(stored.value));
    await context.page.reload({ waitUntil: "domcontentloaded" });
}
