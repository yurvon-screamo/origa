/* eslint-disable @typescript-eslint/no-empty-object-pattern */
import { test as base, type Page } from "@playwright/test";
import {
	getAdminToken,
	createTestUser,
	deleteTestUser,
} from "./admin";
import { trailBaseUrl } from "../config";

export const DEFAULT_TEST_PASSWORD = "e2e-test-password-123";

export function generateUniqueEmail(): string {
	const timestamp = Date.now();
	const random = Math.random().toString(36).substring(2, 8);
	return `e2e-${timestamp}-${random}@origa.local`;
}

export interface AuthFixture {
	testUserEmail: string;
	testUserPassword: string;
}

/**
 * Base test fixture without auth (for tests that don't need a user)
 */
export const test = base.extend<AuthFixture>({
	testUserEmail: async ({}, use) => {
		await use("");
	},
	testUserPassword: async ({}, use) => {
		await use("");
	},
});

export interface UniqueUserFixture extends AuthFixture {
	authToken: string;
	page: Page;
}

/**
 * Fixture that manages unique test user lifecycle
 * Creates a new user before each test, deletes after test
 * Uses UI login for reliable authentication
 */
export const testWithUniqueUser = base.extend<UniqueUserFixture>({
	testUserEmail: async ({}, use) => {
		const email = generateUniqueEmail();
		await use(email);
	},

	testUserPassword: async ({}, use) => {
		await use(DEFAULT_TEST_PASSWORD);
	},

	page: async ({ browser }, use) => {
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
			console.error("[fixture] Failed to create test user:", error);
			throw error;
		}

		try {
			await page.goto("http://localhost:1420");
			await page.locator('input[type="email"], input[data-testid="email-input"]').waitFor({ state: "visible", timeout: 30_000 });

			await page.fill('input[type="email"], input[data-testid="email-input"]', userEmail);
			await page.fill('input[type="password"], input[data-testid="password-input"]', userPassword);
			await page.click('button[type="submit"], button[data-testid="login-submit"]');
			
			await page.waitForURL("**/home**", { timeout: 30_000 }).catch(() => {});
			await page.waitForTimeout(2000);

			await use(page);
		} catch (error) {
			console.error("[fixture] Error during setup:", error);
			throw error;
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

	authToken: async ({}, use) => {
		await use("");
	},
});
