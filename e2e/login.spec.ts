import { expect, test } from "@playwright/test";
import { HomePage, LoginPage, ProfilePage } from "./pages";

const TEST_EMAIL_PREFIX = "test-origa-";
const TEST_PASSWORD = "TestPass123!";
const generateTestEmail = () => `${TEST_EMAIL_PREFIX}${Date.now()}@test.com`;

test.describe("Регистрация и вход в приложение", () => {
	let loginPage: LoginPage;
	let _homePage: HomePage;
	let _profilePage: ProfilePage;
	let testEmail: string;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		_homePage = new HomePage(page);
		_profilePage = new ProfilePage(page);
		testEmail = generateTestEmail();
	});

	test("должен зарегистрировать нового пользователя и показать форму подтверждения email", async ({
		page,
	}) => {
		await loginPage.goto();

		await loginPage.register(testEmail, TEST_PASSWORD);

		await loginPage.expectEmailConfirmationVisible();
		await expect(page.getByText(testEmail)).toBeVisible();
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

		await loginPage.fillEmail(testEmail);
		await loginPage.loginButton.click({ force: true });

		await loginPage.expectErrorMessage("Введите пароль");
		await expect(page).toHaveURL("/");
	});

	test("должен показать ошибку при коротком пароле в форме логина", async ({
		page,
	}) => {
		await loginPage.goto();

		await loginPage.fillEmail(testEmail);
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

	test("должен позволить регистрацию с Enter", async ({ page }) => {
		await loginPage.goto();

		await loginPage.switchToRegister();
		await loginPage.fillEmail(testEmail);
		await loginPage.fillPassword(TEST_PASSWORD);
		await loginPage.passwordInput.press("Enter");

		// После регистрации показывается форма подтверждения email
		await loginPage.expectEmailConfirmationVisible();
	});

	test("должен показать форму подтверждения email при попытке входа с неподтверждённым email", async ({
		page,
	}) => {
		// Сначала регистрируем
		await loginPage.goto();
		await loginPage.register(testEmail, TEST_PASSWORD);
		await loginPage.expectEmailConfirmationVisible();

		// Возвращаемся к входу
		await page
			.getByRole("button", { name: "Вернуться к входу" })
			.click({ force: true });
		await expect(loginPage.loginButton).toBeVisible();

		// Пытаемся войти - должно показать форму подтверждения email
		await loginPage.login(testEmail, TEST_PASSWORD);
		await loginPage.expectEmailConfirmationVisible();
	});

	test("должен показать форму подтверждения при регистрации уже зарегистрированного email", async ({
		page,
	}) => {
		// Регистрируем первый раз
		await loginPage.goto();
		await loginPage.register(testEmail, TEST_PASSWORD);
		await loginPage.expectEmailConfirmationVisible();

		// Возвращаемся и пробуем зарегистрировать тот же email снова
		await page
			.getByRole("button", { name: "Вернуться к входу" })
			.click({ force: true });
		await loginPage.switchToRegister();
		await loginPage.fillEmail(testEmail);
		await loginPage.fillPassword(TEST_PASSWORD);
		await loginPage.registerButton.click({ force: true });

		// Должна показаться форма подтверждения email (т.к. email уже зарегистрирован, но не подтверждён)
		await loginPage.expectEmailConfirmationVisible();
	});
});

test.describe("Повторная отправка письма подтверждения", () => {
	let loginPage: LoginPage;
	let testEmail: string;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		testEmail = generateTestEmail();
	});

	test("должен позволить повторно отправить письмо подтверждения", async ({
		page,
	}) => {
		await loginPage.goto();
		await loginPage.register(testEmail, TEST_PASSWORD);
		await loginPage.expectEmailConfirmationVisible();

		// Нажимаем кнопку повторной отправки
		await loginPage.resendEmailButton.click({ force: true });

		// Должно появиться сообщение об успешной отправке
		await expect(page.locator(".bg-green-950\\/20")).toContainText(
			"Письмо отправлено повторно",
		);
	});
});
