import { expect, test } from "@playwright/test";
import { LoginPage } from "../pages";

test.describe("UJ6: Валидация формы входа/регистрации", () => {
	let loginPage: LoginPage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		await loginPage.goto();
	});

	test("ошибка при пустом email", async () => {
		await loginPage.fillPassword("TestPass123!");
		await loginPage.loginButton.click({ force: true });

		await loginPage.expectErrorMessage("Введите email");
	});

	test("ошибка при некорректном email", async () => {
		await loginPage.fillEmail("invalid-email");
		await loginPage.fillPassword("TestPass123!");
		await loginPage.loginButton.click({ force: true });

		await loginPage.expectErrorMessage("Некорректный формат email");
	});

	test("ошибка при пустом пароле", async () => {
		await loginPage.fillEmail("test@example.com");
		await loginPage.loginButton.click({ force: true });

		await loginPage.expectErrorMessage("Введите пароль");
	});

	test("ошибка при коротком пароле", async () => {
		await loginPage.fillEmail("test@example.com");
		await loginPage.fillPassword("1234567");
		await loginPage.loginButton.click({ force: true });

		await loginPage.expectErrorMessage("Пароль должен быть минимум 8 символов");
	});

	test("переключение между формами логина и регистрации", async ({ page }) => {
		await expect(loginPage.loginButton).toBeVisible();
		await expect(loginPage.registerButton).not.toBeVisible();

		await loginPage.switchToRegister();
		await expect(loginPage.registerButton).toBeVisible();
		await expect(loginPage.loginButton).not.toBeVisible();

		await loginPage.switchToLogin();
		await expect(loginPage.loginButton).toBeVisible();
		await expect(loginPage.registerButton).not.toBeVisible();
	});
});
