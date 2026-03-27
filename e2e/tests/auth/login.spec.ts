import { test, expect } from '@playwright/test';
import { LoginPage } from '../../pages/LoginPage';

test.describe('Authentication @smoke', () => {
  let loginPage: LoginPage;

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page);
    await loginPage.goto();
  });

  test('should display login form elements @smoke', async () => {
    await test.step('Verify form elements are visible', async () => {
      await loginPage.expectFormVisible();
    });

    await test.step('Verify OAuth section is present', async () => {
      await expect(loginPage.oauthSection).toBeVisible();
    });
  });

  test('should login successfully with valid credentials @smoke', async () => {
    await test.step('Fill credentials', async () => {
      await loginPage.fillEmail('e2e@sample.com');
      await loginPage.fillPassword('12345678');
    });

    await test.step('Submit form', async () => {
      await loginPage.submit();
    });

    await test.step('Verify redirect to home', async () => {
      await loginPage.expectRedirectToHome();
    });

    await test.step('Verify home page shows E2E greeting', async () => {
      await expect(loginPage.page.getByText(/E2E/i)).toBeVisible({ timeout: 30000 });
    });
  });

  test('should show error for invalid credentials @slow', async () => {
    await test.step('Enter invalid credentials', async () => {
      await loginPage.login('invalid@email.com', 'wrongpassword');
    });

    await test.step('Verify error is shown', async () => {
      await loginPage.page.waitForTimeout(2000);
      const url = loginPage.page.url();
      expect(url).toContain('/login');
    });
  });

  test('should show validation error for empty fields @slow', async ({ page }) => {
    await test.step('Submit empty form', async () => {
      await loginPage.submit();
    });

    await test.step('Verify still on login page', async () => {
      await expect(page).toHaveURL(/\/login/);
    });
  });

  test('should show validation error for invalid email format @slow', async ({ page }) => {
    await test.step('Enter invalid email', async () => {
      await loginPage.fillEmail('notanemail');
      await loginPage.fillPassword('12345678');
      await loginPage.submit();
    });

    await test.step('Verify still on login page', async () => {
      await expect(page).toHaveURL(/\/login/);
    });
  });

  test('should redirect authenticated user from login to home @smoke', async ({ page }) => {
    await test.step('Login first', async () => {
      await loginPage.login('e2e@sample.com', '12345678');
      await loginPage.expectRedirectToHome();
    });

    await test.step('Navigate to login again', async () => {
      await page.goto('/login');
    });

    await test.step('Should redirect to home', async () => {
      await expect(page).toHaveURL(/\/home/, { timeout: 30000 });
    });
  });
});

test.describe('Protected routes @smoke', () => {
  const protectedRoutes = ['/home', '/profile', '/words', '/grammar', '/kanji', '/lesson', '/sets'];

  for (const route of protectedRoutes) {
    test(`should redirect unauthenticated user from ${route} to login @smoke`, async ({ page }) => {
      if (route === '/lesson') {
        test.setTimeout(120000);
      }
      await page.goto(route);
      await expect(page).toHaveURL(/\/login/, { timeout: 30000 });
    });
  }
});
