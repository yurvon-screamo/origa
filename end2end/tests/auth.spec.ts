import { test, testWithUniqueUser } from "../fixtures";
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

    test("should show error for invalid credentials", async () => {
        await loginPage.login("invalid@test.com", "wrongpassword");

        await expect(loginPage.errorAlert).toBeVisible({ timeout: 10000 });

        const errorMessage = await loginPage.errorAlert.textContent();
        expect(errorMessage).toBeTruthy();
        expect(errorMessage?.length).toBeGreaterThan(0);
    });
});

testWithUniqueUser.describe("Authentication - success", () => {
    testWithUniqueUser("should login and redirect to onboarding", async ({ page, testUserEmail, testUserPassword }) => {
        const loginPage = new LoginPage(page);
        await loginPage.goto();

        await loginPage.fillEmail(testUserEmail);
        await loginPage.fillPassword(testUserPassword);
        await loginPage.submit();

        await loginPage.expectLoginSuccess(["/onboarding", "/home"]);

        const url = page.url();
        expect(url).toMatch(/\/(onboarding|home)$/);
    });
});
