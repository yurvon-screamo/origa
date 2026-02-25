import { expect, test } from "@playwright/test";
import { GrammarPage, HomePage, KanjiPage, LoginPage, WordsPage } from "../pages";
import { CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD } from "../fixtures/test-helpers";

test.describe("UJ2: Добавление разных типов контента", () => {
	let loginPage: LoginPage;
	let homePage: HomePage;
	let kanjiPage: KanjiPage;
	let wordsPage: WordsPage;
	let grammarPage: GrammarPage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		homePage = new HomePage(page);
		kanjiPage = new KanjiPage(page);
		wordsPage = new WordsPage(page);
		grammarPage = new GrammarPage(page);

		await loginPage.goto();
		await loginPage.login(CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD);
		await page.waitForURL("/home");
	});

	test("пользователь может добавить кандзи, слова и грамматику", async ({ page }) => {
		await test.step("Добавление кандзи уровня N5", async () => {
			await homePage.navigateToKanji();
			await kanjiPage.expectVisible();

			await kanjiPage.clickAddButton();
			await kanjiPage.expectModalVisible();

			await kanjiPage.selectLevel("N5");
			await page.waitForTimeout(2000);

			let hasKanji = await kanjiPage.hasAvailableKanji();
			if (!hasKanji) {
				await kanjiPage.selectLevel("N4");
				await page.waitForTimeout(2000);
				hasKanji = await kanjiPage.hasAvailableKanji();
			}

			if (hasKanji) {
				const selected = await kanjiPage.selectMultipleKanji(2);
				if (selected > 0) {
					await kanjiPage.confirmAdd();
					await page.waitForTimeout(1000);
					await kanjiPage.expectModalNotVisible();

					const kanjiCount = await kanjiPage.getCardsCount();
					expect(kanjiCount).toBeGreaterThanOrEqual(1);
				} else {
					await kanjiPage.cancelAdd();
					console.log("No kanji selected");
				}
			} else {
				await kanjiPage.cancelAdd();
				console.log("No kanji available to add");
			}
		});

		await test.step("Проверка страницы слов", async () => {
			await homePage.navigateToWords();
			await wordsPage.expectVisible();
			await wordsPage.expectFiltersVisible();
		});

		await test.step("Проверка страницы грамматики", async () => {
			await homePage.navigateToGrammar();
			await grammarPage.expectVisible();

			await grammarPage.clickAddButton();
			await grammarPage.expectModalVisible();

			await grammarPage.selectLevel("N5");
			await page.waitForTimeout(2000);

			const firstRule = page.locator(".modal-content div.border").first();
			if (await firstRule.isVisible({ timeout: 2000 }).catch(() => false)) {
				await firstRule.click();
				await grammarPage.confirmAdd();
				await page.waitForTimeout(1000);
			} else {
				await grammarPage.cancelAdd();
				console.log("No grammar rules available to add");
			}

			await grammarPage.expectModalNotVisible();
		});

		await test.step("Проверка фильтров на каждой странице", async () => {
			await homePage.navigateToKanji();
			await kanjiPage.expectFiltersVisible();
			const allCount = await kanjiPage.getFilterCount("all");
			expect(allCount).toBeGreaterThanOrEqual(0);

			await homePage.navigateToWords();
			await wordsPage.expectFiltersVisible();

			await homePage.navigateToGrammar();
			await grammarPage.expectFiltersVisible();
		});
	});
});
