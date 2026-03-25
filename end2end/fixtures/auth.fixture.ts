import { test as base } from "@playwright/test";
import {
	getAdminToken,
	createTestUser,
	deleteTestUser,
	loginTestUser,
} from "./admin";

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
	testUserEmail: async (_, use) => {
		await use("");
	},
	testUserPassword: async (_, use) => {
		await use("");
	},
});

export interface UniqueUserFixture extends AuthFixture {
	authToken: string;
}

/**
 * Fixture that manages unique test user lifecycle
 * Creates a new user before each test, deletes after test
 * Provides authToken for authenticated requests
 */
export const testWithUniqueUser = base.extend<UniqueUserFixture>({
	testUserEmail: async (_, use) => {
		const email = generateUniqueEmail();
		await use(email);
	},

	testUserPassword: async (_, use) => {
		await use(DEFAULT_TEST_PASSWORD);
	},

	authToken: async ({ testUserEmail, testUserPassword }, use) => {
		let adminToken: string | undefined;

		try {
			adminToken = await getAdminToken();
			await createTestUser(adminToken, testUserEmail, testUserPassword);

			const { token: authToken } = await loginTestUser(testUserEmail, testUserPassword);
			await use(authToken);
		} catch (error) {
			console.error("[fixture] Failed to setup test user:", error);
			throw error;
		} finally {
			if (adminToken) {
				try {
					await deleteTestUser(adminToken, testUserEmail);
				} catch (error) {
					console.error("[fixture] Failed to cleanup test user");
				}
			}
		}
	},
});
