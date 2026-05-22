/**
 * Tauri Desktop Test Setup
 *
 * Tauri tests require a built desktop application.
 * They are skipped in CI by default and must be run locally.
 */

import type { Page } from "@playwright/test";

type BrowserGlobalThis = typeof globalThis & { __TAURI__: Record<string, unknown> };

export const isTauriContext = (): boolean => {
    return process.env.TAURI_CONTEXT === "true";
};

export const skipInCI = process.env.CI !== undefined;

export async function connectToTauriWindow(page: Page): Promise<void> {
    await page.waitForFunction(
        () => {
            return (globalThis as BrowserGlobalThis).__TAURI__ !== undefined;
        },
        { timeout: 30000 },
    );
}

export async function getTauriApi(page: Page): Promise<Record<string, unknown>> {
    await connectToTauriWindow(page);
    return await page.evaluate(() => (globalThis as BrowserGlobalThis).__TAURI__);
}
