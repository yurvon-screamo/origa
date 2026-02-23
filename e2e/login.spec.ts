import { expect, test } from "@playwright/test";
import { LoginPage } from "./pages";

const TEST_PASSWORD = "TestPass123!";

test.describe("Валидация формы входа", () => {
	let loginPage: LoginPage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
	});

	test("должен показать ошибку при пустом email в форме логина", async ({
		page,
	}) => {
		await loginPage.goto();

		await loginPage.fillPassword(TEST_PASSWORD);
		await loginPage.loginButton.click({ force: true });

		await loginPage.expectErrorMessage("Введите email");
		await expect(page).toHaveURL("/");
	});

	test("должен показать ошибку при пустом пароле в форме логина", async ({
		page,
	}) => {
		await loginPage.goto();

		await loginPage.fillEmail("test@example.com");
		await loginPage.loginButton.click({ force: true });

		await loginPage.expectErrorMessage("Введите пароль");
		await expect(page).toHaveURL("/");
	});

	test("должен показать ошибку при коротком пароле в форме логина", async ({
		page,
	}) => {
		await loginPage.goto();

		await loginPage.fillEmail("test@example.com");
		await loginPage.fillPassword("1234567");
		await loginPage.loginButton.click({ force: true });

		await loginPage.expectErrorMessage("Пароль должен быть минимум 8 символов");
		await expect(page).toHaveURL("/");
	});

	test("должен показать ошибку при некорректном email в форме логина", async ({
		page,
	}) => {
		await loginPage.goto();

		await loginPage.fillEmail("invalid-email");
		await loginPage.fillPassword(TEST_PASSWORD);
		await loginPage.loginButton.click({ force: true });

		await loginPage.expectErrorMessage("Некорректный формат email");
		await expect(page).toHaveURL("/");
	});

	test("должен переключаться между формами логина и регистрации", async ({
		page,
	}) => {
		await loginPage.goto();

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
