import { expect, test } from "@playwright/test";
import { HomePage, LoginPage, ProfilePage } from "./pages";

const CONFIRMED_TEST_EMAIL = process.env.E2E_TEST_EMAIL;
const CONFIRMED_TEST_PASSWORD = process.env.E2E_TEST_PASSWORD;

test.describe("Страница профиля", () => {
	let loginPage: LoginPage;
	let _homePage: HomePage;
	let profilePage: ProfilePage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		_homePage = new HomePage(page);
		profilePage = new ProfilePage(page);

		// Login with pre-confirmed test user
		await loginPage.goto();
		await loginPage.login(CONFIRMED_TEST_EMAIL!, CONFIRMED_TEST_PASSWORD!);
		await page.waitForURL("/home");
	});

	test("должен отобразить все элементы страницы профиля", async ({ page }) => {
		await profilePage.goto();
		await profilePage.expectVisible();
	});

	test("должен отобразить email в заголовке", async ({ page }) => {
		await profilePage.goto();
		// Email prefix (before @) should be in the heading
		await profilePage.expectHeadingContains(
			CONFIRMED_TEST_EMAIL?.split("@")[0],
		);
	});

	test("должен отобразить email в поле ввода", async ({ page }) => {
		await profilePage.goto();
		await profilePage.expectUsername(CONFIRMED_TEST_EMAIL?.split("@")[0]);
	});

	test("должен позволить изменить Duolingo токен", async ({ page }) => {
		await profilePage.goto();
		const newToken = "test-token-12345";

		await profilePage.setDuolingoToken(newToken);
		await profilePage.expectDuolingoToken(newToken);
	});

	test("должен позволить переключить напоминания", async ({ page }) => {
		await profilePage.goto();

		await profilePage.toggleReminders();
		await profilePage.expectRemindersEnabled(false);
	});

	test("должен сохранить изменения", async ({ page }) => {
		await profilePage.goto();
		const newToken = "new-duolingo-token";

		await profilePage.setDuolingoToken(newToken);
		await profilePage.saveChanges();
		await profilePage.expectDuolingoToken(newToken);
	});

	test("должен выйти из аккаунта", async ({ page }) => {
		await profilePage.goto();
		await profilePage.logout();

		await expect(page).toHaveURL("/");
		await loginPage.expectVisible();
	});

	test("должен показать диалог подтверждения удаления аккаунта", async ({
		page,
	}) => {
		await profilePage.goto();

		// Click delete account button
		await profilePage.deleteAccountButton.click();

		// Verify confirmation dialog is shown
		await expect(profilePage.confirmDeleteButton).toBeVisible();
		await expect(profilePage.cancelDeleteButton).toBeVisible();
	});

	test("должен отменить удаление аккаунта", async ({ page }) => {
		await profilePage.goto();

		await profilePage.cancelDelete();

		// Should still be on profile page
		await expect(page).toHaveURL("/profile");
		await profilePage.expectVisible();
	});
});
