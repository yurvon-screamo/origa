import { type Page } from "@playwright/test";

/**
 * Waits for the scoring step to finish loading.
 * Resolves when either scoring-step-hint (cards ready) or
 * scoring-step-complete (0 new cards) is visible.
 * Throws if neither appears within timeout.
 */
export async function waitForScoringReady(page: Page, timeout = 30_000): Promise<void> {
    await Promise.race([
        page.getByTestId("scoring-step-hint").waitFor({ state: "visible", timeout }),
        page.getByTestId("scoring-step-complete").waitFor({ state: "visible", timeout }),
    ]);
}
