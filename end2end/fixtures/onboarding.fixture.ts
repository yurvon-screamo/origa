/* eslint-disable @typescript-eslint/no-empty-object-type */
import { test as base, type Page } from "@playwright/test";
import { setupTestUser, withAuthenticatedPage } from "../helpers/auth";
import { loginTestUser } from "./admin";

/**
 * Extended test with authenticated page fixture
 * Manages test user lifecycle and authentication
 */
export const test = base.extend<{
	authenticatedPage: Page;
	authToken: string;
}>({
	authenticatedPage: async ({ browser }, use) => {
		await withAuthenticatedPage(browser, use);
	},

	authToken: async ({}, use) => {
		const userCtx = await setupTestUser();

		try {
			const { token } = await loginTestUser(userCtx.email, userCtx.password);
			await use(token);
		} finally {
			await userCtx.cleanup();
		}
	},
});

/**
 * Fixture for tests that need a fresh user (no completed onboarding)
 * Uses UI login for reliability
 */
export const testWithFreshUser = base.extend<{
	page: Page;
}>({
	page: async ({ browser }, use) => {
		await withAuthenticatedPage(browser, use);
	},
});
