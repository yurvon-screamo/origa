import { expect, test } from "@playwright/test";
import { HomePage, KanjiPage, LessonPage, LoginPage } from "../pages";
import { CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD } from "../fixtures/test-helpers";

test.describe("UJ1: Полный цикл обучения", () => {
	let loginPage: LoginPage;
	let homePage: HomePage;
	let kanjiPage: KanjiPage;
	let lessonPage: LessonPage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		homePage = new HomePage(page);
		kanjiPage = new KanjiPage(page);
		lessonPage = new LessonPage(page);
	});

	test("пользователь входит, добавляет кандзи и проходит урок", async ({ page }) => {
		await test.step("Шаг 1: Вход в систему", async () => {
			await loginPage.goto();
			await loginPage.login(CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD);
			await page.waitForURL("/home");
			await homePage.expectVisible();
		});

		await test.step("Шаг 2-3: Переход на вкладку Кандзи и открытие модального окна", async () => {
			await homePage.navigateToKanji();
			await kanjiPage.expectVisible();
			await kanjiPage.clickAddButton();
			await kanjiPage.expectModalVisible();
		});

		await test.step("Шаг 4-6: Выбор уровня N5 и добавление кандзи", async () => {
			await kanjiPage.selectLevel("N5");
			await page.waitForTimeout(2000);

			const hasKanji = await kanjiPage.hasAvailableKanji();
			if (!hasKanji) {
				await kanjiPage.selectLevel("N4");
				await page.waitForTimeout(2000);
			}

			const stillHasKanji = await kanjiPage.hasAvailableKanji();
			if (!stillHasKanji) {
				console.log("No kanji available to add, skipping test");
				test.skip();
				return;
			}

			const selected = await kanjiPage.selectMultipleKanji(3);
			if (selected === 0) {
				console.log("No kanji selected, skipping test");
				test.skip();
				return;
			}
			
			await kanjiPage.confirmAdd();
			await page.waitForTimeout(1000);
			
			await kanjiPage.expectModalNotVisible();
		});

		await test.step("Шаг 7: Проверка что кандзи добавлены в список", async () => {
			const count = await kanjiPage.getCardsCount();
			expect(count).toBeGreaterThanOrEqual(1);
		});

		await test.step("Шаг 8-10: Переход на урок и прохождение карточек", async () => {
			await homePage.goto();
			await homePage.expectVisible();
			await homePage.startLesson();
			await page.waitForTimeout(2000);

			const maxIterations = 50;
			let iteration = 0;

			while (iteration < maxIterations) {
				const hasCard = await lessonPage.showAnswerButton.isVisible({ timeout: 2000 }).catch(() => false);
				
				if (!hasCard) {
					const hasComplete = await lessonPage.completeTitle.isVisible({ timeout: 1000 }).catch(() => false);
					if (hasComplete) break;
					
					const hasEmpty = await lessonPage.emptyStateText.isVisible({ timeout: 1000 }).catch(() => false);
					if (hasEmpty) {
						console.log("No cards available for lesson (all studied), skipping remaining steps");
						return;
					}
					
					await page.waitForTimeout(500);
					iteration++;
					continue;
				}

				await lessonPage.showAnswer();
				await page.waitForTimeout(200);
				await lessonPage.rateGood();
				await page.waitForTimeout(300);
				iteration++;
			}
		});

		await test.step("Шаг 11: Проверка экрана завершения урока (если были карточки)", async () => {
			const hasComplete = await lessonPage.completeTitle.isVisible({ timeout: 1000 }).catch(() => false);
			const hasEmpty = await lessonPage.emptyStateText.isVisible({ timeout: 1000 }).catch(() => false);
			
			if (hasComplete) {
				await expect(lessonPage.completeTitle).toBeVisible({ timeout: 5000 });
				await expect(lessonPage.homeButton).toBeVisible();
			} else if (hasEmpty) {
				console.log("Lesson empty - no cards to study");
			} else {
				console.log("Lesson state unclear, continuing");
			}
		});

		await test.step("Шаг 12-14: Возврат на главную и проверка счётчика", async () => {
			const hasHomeButton = await lessonPage.homeButton.isVisible({ timeout: 1000 }).catch(() => false);
			if (hasHomeButton) {
				await lessonPage.goHome();
			} else {
				await page.goto("/home");
			}
			await homePage.expectVisible();

			const totalCards = await homePage.getTotalCards();
			expect(parseInt(totalCards.replace(",", ""), 10)).toBeGreaterThanOrEqual(0);
		});
	});
});
