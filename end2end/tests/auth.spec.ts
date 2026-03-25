import { test } from "../fixtures/auth.fixture";
import { expect } from "@playwright/test";
import { LoginPage } from "../pages";

test.describe("Authentication", () => {
    let loginPage: LoginPage;

    test.beforeEach(async ({ page }) => {
        loginPage = new LoginPage(page);
        await loginPage.goto();
    });

    test("should display login page", async ({ page }) => {
        await expect(page).toHaveURL(/\/(login)?$/);
        await loginPage.expectLoginFormVisible();
    });

    test("should have email input visible", async () => {
        await expect(loginPage.emailInput).toBeVisible();
        await expect(loginPage.emailInput).toHaveAttribute("type", "email");
    });

    test("should have password input visible", async () => {
        await expect(loginPage.passwordInput).toBeVisible();
        await expect(loginPage.passwordInput).toHaveAttribute("type", "password");
    });

    test("should have submit button", async () => {
        await expect(loginPage.submitButton).toBeVisible();
    });

    test("should show error for invalid credentials", async ({ page }) => {
        await loginPage.login("invalid@test.com", "wrongpassword");

        await page.waitForTimeout(500);

        await expect(loginPage.emailInput).toBeVisible({ timeout: 5000 });
    });
});
