/* eslint-disable @typescript-eslint/no-empty-object-type */
import { test as base, type Page } from "@playwright/test";
import { setupTestUser, withAuthenticatedPage } from "../helpers/auth";

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
		const userCtx = await setupTestUser();
		await use(userCtx.email);
		await userCtx.cleanup();
	},

	testUserPassword: async ({}, use) => {
		await use("");
	},

	page: async ({ browser }, use) => {
		await withAuthenticatedPage(browser, use);
	},

	authToken: async ({}, use) => {
		await use("");
	},
});
