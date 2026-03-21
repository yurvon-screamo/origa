import { test as setup, expect } from '@playwright/test';
import path from 'path';
import fs from 'fs';

const authFile = path.join(__dirname, '..', 'playwright', '.auth', 'user.json');

setup('authenticate', async ({ page }) => {
  setup.setTimeout(120000);

  await setup.step('Navigate to login page', async () => {
    await page.goto('/login');
    await expect(page.getByRole('heading', { name: /オリガ/i })).toBeVisible({ timeout: 30000 });
  });

  await setup.step('Fill email field', async () => {
    const emailInput = page.getByPlaceholder(/example@mail.com/i);
    await expect(emailInput).toBeVisible({ timeout: 30000 });
    await emailInput.fill('e2e@sample.com');
  });

  await setup.step('Fill password field', async () => {
    const passwordInput = page.locator('input[type="password"]');
    await expect(passwordInput).toBeVisible({ timeout: 30000 });
    await passwordInput.fill('12345678');
  });

  await setup.step('Submit login form', async () => {
    const submitButton = page.getByRole('button', { name: 'Войти', exact: true });
    await expect(submitButton).toBeVisible({ timeout: 30000 });
    await submitButton.click();
    await page.waitForURL('**/home', { timeout: 30000 });
  });

  await setup.step('Verify authentication successful', async () => {
    await expect(page).toHaveURL(/.*home.*/);
    await expect(page.getByText(/статистика/i)).toBeVisible({ timeout: 30000 });
  });

  await setup.step('Save authentication state', async () => {
    const dir = path.dirname(authFile);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
    await page.context().storageState({ path: authFile });
  });
});
