import type { Browser, BrowserContext, Page } from "@playwright/test";

import { createTestUser, deleteTestUserWithRetry, getAdminToken } from "../fixtures/admin.ts";
import { DEFAULT_TEST_PASSWORD, generateUniqueEmail, uiLogin } from "../helpers/auth.ts";
import { presets } from "./presets/index.ts";
import type { PresetId } from "./presets/registry.ts";
import { VIEWPORTS, type ViewportTag } from "./viewports.ts";

export interface ShotSessionOptions {
    browser: Browser;
    baseURL: string;
    locale: string;
    viewport: ViewportTag;
    preset: PresetId;
}

export interface ShotSession {
    context: BrowserContext;
    page: Page;
    /** Removes the test account and closes the browser context. */
    close(): Promise<void>;
}

/**
 * Opens a logged-in session for a capture.
 *
 * Every screen of every scenario starts from here: a fresh test account,
 * the download consent pre-approved, the login done and the data state of
 * the preset applied. The account is deleted afterwards — a capture must
 * not leave accounts behind.
 */
export async function openShotSession(options: ShotSessionOptions): Promise<ShotSession> {
    const viewport = VIEWPORTS[options.viewport];
    const context = await options.browser.newContext({
        baseURL: options.baseURL,
        locale: options.locale,
        viewport: { width: viewport.width, height: viewport.height },
    });
    const page = await context.newPage();

    const { token, csrfToken } = await getAdminToken();
    const email = generateUniqueEmail();
    const userId = await createTestUser(token, csrfToken, email, DEFAULT_TEST_PASSWORD);

    try {
        await uiLogin(page, email, DEFAULT_TEST_PASSWORD, { baseURL: options.baseURL });
        await presets[options.preset]({ page, baseURL: options.baseURL });
    } catch (error) {
        await context.close();
        await deleteTestUserWithRetry(token, csrfToken, userId, email).catch(() => undefined);
        throw error;
    }

    return {
        context,
        page,
        close: async () => {
            await context.close();
            await deleteTestUserWithRetry(token, csrfToken, userId, email).catch(() => undefined);
        },
    };
}
