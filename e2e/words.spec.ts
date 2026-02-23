import { expect, test } from "@playwright/test";
import { LoginPage, ProfilePage, WordsPage } from "./pages";

const CONFIRMED_TEST_EMAIL = process.env.E2E_TEST_EMAIL;
const CONFIRMED_TEST_PASSWORD = process.env.E2E_TEST_PASSWORD;

test.describe("Страница слов", () => {
	let loginPage: LoginPage;
	let wordsPage: WordsPage;
	let _profilePage: ProfilePage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		wordsPage = new WordsPage(page);
		_profilePage = new ProfilePage(page);

		// Login with pre-confirmed test user
		await loginPage.goto();
		await loginPage.login(CONFIRMED_TEST_EMAIL!, CONFIRMED_TEST_PASSWORD!);
		await page.waitForURL("/home");
	});

	test("должен отобразить все элементы страницы слов", async ({ page }) => {
		await wordsPage.goto();
		await wordsPage.expectVisible();
		await wordsPage.expectFiltersVisible();
	});

	test("должен отобразить список карточек слов (пустой для нового пользователя)", async ({
		page,
	}) => {
		await wordsPage.goto();
		// New user has no words - expect empty state
		await wordsPage.expectEmptyState();
	});

	test("должен показать пустое состояние при отсутствии результатов", async ({
		page,
	}) => {
		await wordsPage.goto();

		await wordsPage.search("несуществующееслово12345");
		await wordsPage.expectEmptyState();
	});

	test("должен переключаться между фильтрами", async ({ page }) => {
		await wordsPage.goto();

		await wordsPage.clickFilter("new");
		await wordsPage.expectFilterActive("new");

		await wordsPage.clickFilter("hard");
		await wordsPage.expectFilterActive("hard");

		await wordsPage.clickFilter("all");
		await wordsPage.expectFilterActive("all");
	});

	test("должен вернуться на главную страницу", async ({ page }) => {
		await wordsPage.goto();
		await wordsPage.goBack();
		await expect(page).toHaveURL("/home");
	});
});
