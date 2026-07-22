import { test, testWithUniqueUser } from "../fixtures";
import { expect } from "@playwright/test";
import { LoginPage } from "../pages";

test.describe("Authentication", () => {
    let loginPage: LoginPage;

    test.beforeEach(async ({ page }) => {
        loginPage = new LoginPage(page);
        await loginPage.goto();
        // The password form is collapsed by default; most auth tests assert on
        // the email/password inputs directly, so expand once for the suite.
        // Tests that care about the collapsed state would opt out explicitly.
        await loginPage.expandPasswordForm();
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

    test("should show error for invalid credentials", async () => {
        await loginPage.login("invalid@test.com", "wrongpassword");

        await expect(loginPage.errorAlert).toBeVisible({ timeout: 10000 });

        const errorMessage = await loginPage.errorAlert.textContent();
        expect(errorMessage).toBeTruthy();
        expect(errorMessage?.length).toBeGreaterThan(0);
        expect(errorMessage).not.toContain("Network error");
        expect(errorMessage).not.toContain("fetching");
    });

    test("should toggle password visibility", async () => {
        await expect(loginPage.passwordInput).toHaveAttribute("type", "password");

        await loginPage.togglePasswordVisibility();
        await expect(loginPage.passwordInput).toHaveAttribute("type", "text");

        await loginPage.togglePasswordVisibility();
        await expect(loginPage.passwordInput).toHaveAttribute("type", "password");
    });

    test("should display language toggle on login page", async ({ page }) => {
        await expect(page.getByTestId("login-lang-toggle")).toBeVisible();
    });
});

testWithUniqueUser.describe("Authentication - success", () => {
    testWithUniqueUser("should have authenticated session", async ({ page }) => {
        const url = page.url();
        const isAuthenticated = url.includes("/home") || url.includes("/onboarding");
        expect(isAuthenticated).toBe(true);
    });
});
