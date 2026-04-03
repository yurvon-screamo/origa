/**
 * Tauri Desktop Test Setup
 *
 * Tauri tests require a built desktop application.
 * They are skipped in CI by default and must be run locally.
 */

import type { Page } from "@playwright/test";

export const isTauriContext = (): boolean => {
    return process.env.TAURI_CONTEXT === "true";
};

export const skipInCI = process.env.CI !== undefined;

export async function connectToTauriWindow(page: Page): Promise<void> {
    await page.waitForFunction(
        () => {
            return (globalThis as any).__TAURI__ !== undefined;
        },
        { timeout: 30000 },
    );
}

export async function getTauriApi(page: Page): Promise<any> {
    await connectToTauriWindow(page);
    return await page.evaluate(() => (globalThis as any).__TAURI__);
}
