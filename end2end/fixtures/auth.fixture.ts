import { test as base, type Page } from '@playwright/test';
import { getAdminToken, createTestUser, deleteTestUser, loginTestUser } from './admin';
import { testUser } from '../config';

export interface AuthFixture {
  testUserEmail: string;
  testUserPassword: string;
}

/**
 * Extended test with auth fixture
 * Provides test user credentials for tests
 */
export const test = base.extend<AuthFixture>({
  testUserEmail: async ({}, use) => {
    await use(testUser.email);
  },
  testUserPassword: async ({}, use) => {
    await use(testUser.password);
  },
});

/**
 * Fixture that manages test user lifecycle
 * Creates user before all tests, deletes after all tests
 * Provides authToken for authenticated requests
 */
export const testWithUser = base.extend<AuthFixture & { page: Page; authToken: string }>({
  testUserEmail: async ({}, use) => {
    await use(testUser.email);
  },
  testUserPassword: async ({}, use) => {
    await use(testUser.password);
  },
  
  page: async ({ page }, use) => {
    await use(page);
  },

  authToken: async ({}, use) => {
    let adminToken: string | undefined;

    try {
      adminToken = await getAdminToken();
      await createTestUser(adminToken);
    } catch (error) {
      console.error('[fixture] Failed to setup test user:', error);
      throw error;
    }

    let authToken = '';
    try {
      // Login as test user to get auth token
      authToken = await loginTestUser();
      await use(authToken);
    } finally {
      // Cleanup: delete test user after all tests in this worker
      if (adminToken) {
        try {
          await deleteTestUser(adminToken);
        } catch (error) {
          // Don't log user email in production
          console.error('[fixture] Failed to cleanup test user');
        }
      }
    }
  },
});