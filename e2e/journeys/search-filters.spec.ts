import { expect, test } from "@playwright/test";
import { GrammarPage, KanjiPage, LoginPage, WordsPage } from "../pages";
import { CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD } from "../fixtures/test-helpers";

test.describe("UJ7: Поиск и фильтрация", () => {
	let loginPage: LoginPage;
	let kanjiPage: KanjiPage;
	let wordsPage: WordsPage;
	let grammarPage: GrammarPage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		kanjiPage = new KanjiPage(page);
		wordsPage = new WordsPage(page);
		grammarPage = new GrammarPage(page);

		await loginPage.goto();
		await loginPage.login(CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD);
		await page.waitForURL("/home");
	});

	test("поиск и фильтрация на странице кандзи", async ({ page }) => {
		await kanjiPage.goto();
		await kanjiPage.expectVisible();
		await kanjiPage.expectFiltersVisible();

		await kanjiPage.search("тестовыйпоиск12345");
		await page.waitForTimeout(500);

		await kanjiPage.expectEmptyState();

		await kanjiPage.clearSearch();
		
		const count = await kanjiPage.getCardsCount();
		expect(count).toBeGreaterThanOrEqual(0);
	});

	test("переключение фильтров на странице слов", async ({ page }) => {
		await wordsPage.goto();
		await wordsPage.expectVisible();
		await wordsPage.expectFiltersVisible();

		await wordsPage.clickFilter("new");
		await wordsPage.expectFilterActive("new");

		await wordsPage.clickFilter("hard");
		await wordsPage.expectFilterActive("hard");

		await wordsPage.clickFilter("all");
		await wordsPage.expectFilterActive("all");
	});

	test("переключение фильтров на странице грамматики", async ({ page }) => {
		await grammarPage.goto();
		await grammarPage.expectVisible();
		await grammarPage.expectFiltersVisible();

		await grammarPage.clickFilter("new");
		await grammarPage.expectFilterActive("new");

		await grammarPage.clickFilter("inProgress");
		await grammarPage.expectFilterActive("inProgress");

		await grammarPage.clickFilter("all");
		await grammarPage.expectFilterActive("all");
	});

	test("поиск на странице грамматики", async ({ page }) => {
		await grammarPage.goto();
		await grammarPage.expectVisible();

		await grammarPage.search("несуществующаяграмматика12345");
		await page.waitForTimeout(500);
		await grammarPage.expectEmptyState();
	});
});
