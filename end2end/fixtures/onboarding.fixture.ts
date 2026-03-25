import { test as base, type Page } from "@playwright/test";
import { LoginPage } from "../pages";
import { testUser, trailBaseUrl } from "../config";
import {
	getAdminToken,
	createTestUser,
	deleteTestUser,
	loginTestUser,
} from "./admin";

/**
 * Extended test with authenticated page fixture
 * Manages test user lifecycle and authentication
 */
export const test = base.extend<{
	authenticatedPage: Page;
	authToken: string;
}>({
	authenticatedPage: async ({ browser }, use) => {
		// Create new browser context
		const context = await browser.newContext();
		const page = await context.newPage();

		// Set viewport for consistent testing
		await page.setViewportSize({ width: 1280, height: 720 });

		let adminToken: string | undefined;

		try {
			// Get admin token
			adminToken = await getAdminToken();
			// Ensure test user exists
			await createTestUser(adminToken);
		} catch (error) {
			console.error("[fixture] Failed to setup test user:", error);
			throw error;
		}

		try {
			// Login to get auth token
			const authToken = await loginTestUser();

			// Navigate to base URL and set auth cookie
			await page.goto(trailBaseUrl);

			// Set auth token in localStorage/sessionStorage if needed
			// This depends on how the app stores auth tokens
			await page.evaluate((token) => {
				localStorage.setItem("auth_token", token);
			}, authToken);

			// Navigate to app
			await page.goto("http://localhost:1420");

			// Wait for auth to complete
			await page.waitForLoadState("networkidle");

			await use(page);
		} finally {
			// Cleanup
			await context.close();
			if (adminToken) {
				try {
					await deleteTestUser(adminToken);
				} catch (error) {
					console.error("[fixture] Failed to cleanup test user");
				}
			}
		}
	},

	authToken: async (_args, use) => {
		const token = await loginTestUser();
		await use(token);
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

		// Create unique test user for this test
		const uniqueEmail = `e2e-fresh-${Date.now()}@origa.local`;
		const uniquePassword = "test-password-123";

		let adminToken: string | undefined;

		try {
			adminToken = await getAdminToken();
			await createTestUser(adminToken, uniqueEmail, uniquePassword);
		} catch (error) {
			console.error("[fixture] Failed to create fresh user:", error);
			throw error;
		}

		try {
			// Login with fresh user
			const authToken = await loginTestUser(uniqueEmail, uniquePassword);

			await page.goto(trailBaseUrl);
			await page.evaluate((token) => {
				localStorage.setItem("auth_token", token);
			}, authToken);

			await page.goto("http://localhost:1420");
			await page.waitForLoadState("networkidle");

			await use(page);
		} finally {
			await context.close();
			if (adminToken) {
				try {
					await deleteTestUser(adminToken, uniqueEmail);
				} catch (error) {
					console.error("[fixture] Failed to cleanup fresh user");
				}
			}
		}
	},
});
