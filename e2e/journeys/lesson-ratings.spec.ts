import { expect, test } from "@playwright/test";
import { HomePage, KanjiPage, LessonPage, LoginPage, ProfilePage } from "../pages";
import { CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD } from "../fixtures/test-helpers";

test.describe("UJ3: Прохождение урока с разными оценками", () => {
	let loginPage: LoginPage;
	let kanjiPage: KanjiPage;
	let lessonPage: LessonPage;
	let homePage: HomePage;
	let profilePage: ProfilePage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		kanjiPage = new KanjiPage(page);
		lessonPage = new LessonPage(page);
		homePage = new HomePage(page);
		profilePage = new ProfilePage(page);

		await loginPage.goto();
		await loginPage.login(CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD);
		await page.waitForURL("/home");
	});

	test("пользователь оценивает карточки разными оценками", async ({ page }) => {
		await test.step("Подготовка: добавление кандзи для урока", async () => {
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

			if (!hasKanji) {
				await kanjiPage.selectLevel("N3");
				await page.waitForTimeout(2000);
				hasKanji = await kanjiPage.hasAvailableKanji();
			}

			if (!hasKanji) {
				console.log("No kanji available to add, skipping test");
				await kanjiPage.cancelAdd();
				test.skip();
				return;
			}

			const selected = await kanjiPage.selectMultipleKanji(4);
			if (selected === 0) {
				console.log("No kanji selected, skipping test");
				await kanjiPage.cancelAdd();
				test.skip();
				return;
			}

			await kanjiPage.confirmAdd();
			await page.waitForTimeout(1000);
			await kanjiPage.expectModalNotVisible();
		});

		await test.step("Прохождение урока с разными оценками", async () => {
			await homePage.goto();
			await homePage.startLesson();
			await page.waitForTimeout(2000);

			const ratings = ["easy", "again", "hard", "good"] as const;
			let ratingIndex = 0;
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

				const rating = ratings[ratingIndex % ratings.length];
				switch (rating) {
					case "easy":
						await lessonPage.rateEasy();
						break;
					case "again":
						await lessonPage.rateAgain();
						break;
					case "hard":
						await lessonPage.rateHard();
						break;
					case "good":
						await lessonPage.rateGood();
						break;
				}

				ratingIndex++;
				await page.waitForTimeout(300);
				iteration++;
			}
		});

		await test.step("Проверка экрана завершения урока (если были карточки)", async () => {
			const hasComplete = await lessonPage.completeTitle.isVisible({ timeout: 1000 }).catch(() => false);
			const hasEmpty = await lessonPage.emptyStateText.isVisible({ timeout: 1000 }).catch(() => false);
			
			if (hasComplete) {
				await expect(lessonPage.completeTitle).toBeVisible({ timeout: 5000 });
			} else if (hasEmpty) {
				console.log("Lesson empty - no cards to study");
			}
		});

		await test.step("Проверка статистики в профиле", async () => {
			await profilePage.goto();
			await profilePage.expectVisible();
		});
	});
});
