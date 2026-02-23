import { expect, test } from "@playwright/test";
import { GrammarPage, LoginPage, ProfilePage } from "./pages";

const CONFIRMED_TEST_EMAIL = process.env.E2E_TEST_EMAIL;
const CONFIRMED_TEST_PASSWORD = process.env.E2E_TEST_PASSWORD;

test.describe("Страница грамматики", () => {
	let loginPage: LoginPage;
	let grammarPage: GrammarPage;
	let _profilePage: ProfilePage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		grammarPage = new GrammarPage(page);
		_profilePage = new ProfilePage(page);

		// Login with pre-confirmed test user
		await loginPage.goto();
		await loginPage.login(CONFIRMED_TEST_EMAIL!, CONFIRMED_TEST_PASSWORD!);
		await page.waitForURL("/home");
	});

	test("должен отобразить все элементы страницы грамматики", async ({
		page,
	}) => {
		await grammarPage.goto();
		await grammarPage.expectVisible();
		await grammarPage.expectFiltersVisible();
	});

	test("должен отобразить сообщение о пустом списке для нового пользователя", async ({
		page,
	}) => {
		await grammarPage.goto();
		await grammarPage.expectEmptyState();
	});

	test("должен показать пустое состояние при отсутствии результатов", async ({
		page,
	}) => {
		await grammarPage.goto();

		await grammarPage.search("несуществующаяграмматика12345");
		await grammarPage.expectEmptyState();
	});

	test("должен переключаться между фильтрами", async ({ page }) => {
		await grammarPage.goto();

		await grammarPage.clickFilter("new");
		await grammarPage.expectFilterActive("new");

		await grammarPage.clickFilter("hard");
		await grammarPage.expectFilterActive("hard");

		await grammarPage.clickFilter("all");
		await grammarPage.expectFilterActive("all");
	});

	test("должен отображать счетчики в фильтрах", async ({ page }) => {
		await grammarPage.goto();

		const allCount = await grammarPage.getFilterCount("all");
		expect(allCount).toBeGreaterThanOrEqual(0);
	});

	test("должен вернуться на главную страницу", async ({ page }) => {
		await grammarPage.goto();
		await grammarPage.goBack();
		await expect(page).toHaveURL("/home");
	});

	test("должен открыть модальное окно добавления грамматики", async ({
		page,
	}) => {
		await grammarPage.goto();
		await grammarPage.clickAddButton();
		await grammarPage.expectModalVisible();
	});

	test("должен закрыть модальное окно по кнопке отмена", async ({ page }) => {
		await grammarPage.goto();
		await grammarPage.clickAddButton();
		await grammarPage.expectModalVisible();
		await grammarPage.cancelAdd();
		await grammarPage.expectModalNotVisible();
	});

	test("должен отображать кнопки выбора уровня JLPT в модальном окне", async ({
		page,
	}) => {
		await grammarPage.goto();
		await grammarPage.clickAddButton();
		await grammarPage.expectModalVisible();

		await expect(
			page.getByRole("button", { name: "N5", exact: true }),
		).toBeVisible();
		await expect(
			page.getByRole("button", { name: "N4", exact: true }),
		).toBeVisible();
		await expect(
			page.getByRole("button", { name: "N3", exact: true }),
		).toBeVisible();
		await expect(
			page.getByRole("button", { name: "N2", exact: true }),
		).toBeVisible();
		await expect(
			page.getByRole("button", { name: "N1", exact: true }),
		).toBeVisible();
	});

	test("должен переключать уровень JLPT в модальном окне", async ({ page }) => {
		await grammarPage.goto();
		await grammarPage.clickAddButton();
		await grammarPage.expectModalVisible();

		await grammarPage.selectLevel("N4");
		await expect(
			page.getByRole("button", { name: "N4", exact: true }),
		).toHaveClass(/btn-olive/);
	});
});
