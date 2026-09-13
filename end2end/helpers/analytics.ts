import { expect, test, type Page } from "@playwright/test";

/**
 * CI-build analytics guard (ADR-054).
 *
 * The dist served by e2e is compiled with `UMAMI_DISABLED=1`, so the Umami
 * tracker must not be present anywhere in the document. This guards the
 * build gate itself: if the CI env ever stops reaching the trunk build
 * (renamed variable, refactored step), CI data silently pollutes production
 * analytics — nothing else would fail.
 *
 * Skipped outside CI: local runs serve a regular trunk build where analytics
 * is enabled by default (mute locally by exporting `UMAMI_DISABLED=1` before
 * `npx playwright test` — the webServer inherits the environment).
 */
export async function expectUmamiTrackerAbsent(page: Page): Promise<void> {
    test.skip(!process.env.CI, "CI-build guard: dist outside CI keeps analytics enabled");
    await expect(page.locator("script[data-website-id]")).toHaveCount(0);
}
