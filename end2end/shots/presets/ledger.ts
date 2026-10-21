import { fresh } from "./fresh.ts";
import { readStoredUser, seedLedgerState, writeStoredUser } from "../seed.ts";
import type { PresetContext } from "./index.ts";

/**
 * A profile with the demo corpus of the insight ledger.
 *
 * The corpus is written from scratch — a small set of Japanese words with
 * the review states the ledger needs — so the screen shows exactly the
 * demo set instead of a real import.
 */
export async function ledger(context: PresetContext): Promise<void> {
    await fresh(context);

    const stored = await readStoredUser(context.page);
    await writeStoredUser(context.page, stored.key, seedLedgerState(stored.value));
    await context.page.reload({ waitUntil: "domcontentloaded" });
}
