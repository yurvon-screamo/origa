import { test as base, type Page } from "@playwright/test";
import { setupTestUser, uiLogin, DEFAULT_TEST_PASSWORD } from "../helpers/auth";
import { loginTestUser } from "./admin";

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
		await use(DEFAULT_TEST_PASSWORD);
	},

	page: async ({ browser, testUserEmail, testUserPassword }, use) => {
		const context = await browser.newContext();
		const page = await context.newPage();
		await page.setViewportSize({ width: 1280, height: 720 });

		try {
			await uiLogin(page, testUserEmail, testUserPassword);
			await use(page);
		} finally {
			await context.close();
		}
	},

	authToken: async ({ testUserEmail, testUserPassword }, use) => {
		const { token } = await loginTestUser(testUserEmail, testUserPassword);
		await use(token);
	},
});
