/* eslint-disable @typescript-eslint/no-empty-object-pattern */
import { test as base, type Page } from "@playwright/test";
import { trailBaseUrl } from "../config";
import {
    getAdminToken,
    createTestUser,
    deleteTestUser,
    loginTestUser,
} from "./admin";
import { generateUniqueEmail, DEFAULT_TEST_PASSWORD } from "./auth.fixture";
import { LoginPage } from "../pages/login.page";

/**
 * Extended test with authenticated page fixture
 * Manages test user lifecycle and authentication
 */
export const test = base.extend<{
    authenticatedPage: Page;
    authToken: string;
}>({
    authenticatedPage: async ({ browser }, use) => {
        const context = await browser.newContext();
        const page = await context.newPage();
        await page.setViewportSize({ width: 1280, height: 720 });

        const userEmail = generateUniqueEmail();
        const userPassword = DEFAULT_TEST_PASSWORD;
        let adminToken: string | undefined;
        let adminCsrfToken: string | undefined;
        let userUuid: string | undefined;

        try {
            const adminAuth = await getAdminToken();
            adminToken = adminAuth.token;
            adminCsrfToken = adminAuth.csrfToken;

            userUuid = await createTestUser(adminToken, adminCsrfToken, userEmail, userPassword);
        } catch (error) {
            console.error("[fixture] Failed to setup test user:", error);
            throw error;
        }

        try {
            const { token: authToken } = await loginTestUser(userEmail, userPassword);

            await page.goto(trailBaseUrl);
            await page.evaluate((token) => {
                localStorage.setItem("auth_token", token);
            }, authToken);

            await page.goto("http://localhost:1420");
            await page.waitForLoadState("networkidle");

            await use(page);
        } finally {
            await context.close();
            if (adminToken && adminCsrfToken && userUuid) {
                try {
                    await deleteTestUser(adminToken, adminCsrfToken, userUuid);
                } catch (error) {
                    console.error("[fixture] Failed to cleanup test user");
                }
            }
        }
    },

    authToken: async ({ }, use) => {
        const userEmail = generateUniqueEmail();
        const userPassword = DEFAULT_TEST_PASSWORD;
        let adminToken: string | undefined;
        let adminCsrfToken: string | undefined;
        let userUuid: string | undefined;

        try {
            const adminAuth = await getAdminToken();
            adminToken = adminAuth.token;
            adminCsrfToken = adminAuth.csrfToken;

            userUuid = await createTestUser(adminToken, adminCsrfToken, userEmail, userPassword);
            const { token } = await loginTestUser(userEmail, userPassword);
            await use(token);
        } finally {
            if (adminToken && adminCsrfToken && userUuid) {
                try {
                    await deleteTestUser(adminToken, adminCsrfToken, userUuid);
                } catch (error) {
                    console.error("[fixture] Failed to cleanup test user");
                }
            }
        }
    },
});

/**
 * Fixture for tests that need a fresh user (no completed onboarding)
 */
export const testWithFreshUser = base.extend<{
    page: Page;
}>({
    page: async ({ browser }, use) => {
        const context = await browser.newContext();
        const page = await context.newPage();
        await page.setViewportSize({ width: 1280, height: 720 });

        const uniqueEmail = generateUniqueEmail();
        const uniquePassword = DEFAULT_TEST_PASSWORD;
        let adminToken: string | undefined;
        let adminCsrfToken: string | undefined;
        let userUuid: string | undefined;

        try {
            const adminAuth = await getAdminToken();
            adminToken = adminAuth.token;
            adminCsrfToken = adminAuth.csrfToken;

            userUuid = await createTestUser(adminToken, adminCsrfToken, uniqueEmail, uniquePassword);
        } catch (error) {
            console.error("[fixture] Failed to create fresh user:", error);
            throw error;
        }

        try {
            // Login through UI using LoginPage
            const loginPage = new LoginPage(page);
            await loginPage.goto();

            const result = await loginPage.login(uniqueEmail, uniquePassword);

            if (!result.success) {
                throw new Error(`Login failed: ${result.error}`);
            }

            // Wait for successful redirect after login
            await loginPage.expectLoginSuccess();

            await use(page);
        }
        catch (error) {
            console.error("[fixture] Failed to login fresh user:", error);
            throw error;
        }
        finally {
            await context.close();
            if (adminToken && adminCsrfToken && userUuid) {
                try {
                    await deleteTestUser(adminToken, adminCsrfToken, userUuid);
                } catch (error) {
                    console.error("[fixture] Failed to cleanup fresh user");
                }
            }
        }
    },
});
