import { expect, test } from "@playwright/test";
import { LoginPage, ProfilePage } from "../pages";
import { CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD } from "../fixtures/test-helpers";

test.describe("UJ5: Работа с профилем", () => {
	let loginPage: LoginPage;
	let profilePage: ProfilePage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		profilePage = new ProfilePage(page);

		await loginPage.goto();
		await loginPage.login(CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD);
		await page.waitForURL("/home");
	});

	test("пользователь может изменить и сохранить настройки профиля", async ({ page }) => {
		await test.step("Открытие профиля", async () => {
			await profilePage.goto();
			await profilePage.expectVisible();
		});

		await test.step("Проверка email в заголовке", async () => {
			const emailPrefix = CONFIRMED_TEST_EMAIL.split("@")[0];
			await profilePage.expectHeadingContains(emailPrefix);
		});

		await test.step("Переключение напоминаний", async () => {
			const initialState = await page.locator('.toggle-container input[type="checkbox"]').isChecked();
			await profilePage.toggleReminders();
		});

		await test.step("Сохранение изменений", async () => {
			await profilePage.saveChanges();
		});

		await test.step("Проверка сохранённых данных после перезагрузки", async () => {
			await page.reload();
			await profilePage.expectVisible();
		});

		await test.step("Выход из аккаунта", async () => {
			await profilePage.logout();
			await expect(page).toHaveURL("/");
			await loginPage.expectVisible();
		});
	});

	test("пользователь может отменить удаление аккаунта", async ({ page }) => {
		await profilePage.goto();
		await profilePage.expectVisible();

		await profilePage.cancelDelete();

		await expect(page).toHaveURL("/profile");
		await profilePage.expectVisible();
	});
});
