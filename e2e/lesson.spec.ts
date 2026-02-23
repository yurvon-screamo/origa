import { expect, test } from "@playwright/test";
import { LessonPage, LoginPage } from "./pages";

const CONFIRMED_TEST_EMAIL = process.env.E2E_TEST_EMAIL;
const CONFIRMED_TEST_PASSWORD = process.env.E2E_TEST_PASSWORD;

test.describe("Страница урока", () => {
	let loginPage: LoginPage;
	let lessonPage: LessonPage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		lessonPage = new LessonPage(page);

		await loginPage.goto();
		await loginPage.login(CONFIRMED_TEST_EMAIL!, CONFIRMED_TEST_PASSWORD!);
		await page.waitForURL("/home");
	});

	test("должен отобразить заголовок страницы урока", async ({ page }) => {
		await lessonPage.goto();
		await lessonPage.expectVisible();
	});

	test("должен отобразить прогресс-бар", async ({ page }) => {
		await lessonPage.goto();
		await page.waitForTimeout(1000);
		await lessonPage.expectProgressBarVisible();
	});

	test("должен отобразить карточку с вопросом", async ({ page }) => {
		await lessonPage.goto();
		await page.waitForTimeout(1000);
		await lessonPage.expectCardVisible();
	});

	test("должен показать ответ при нажатии кнопки", async ({ page }) => {
		await lessonPage.goto();
		await page.waitForTimeout(1000);
		await lessonPage.expectCardVisible();
		await lessonPage.showAnswer();
	});

	test("должен отобразить кнопки оценки после показа ответа", async ({ page }) => {
		await lessonPage.goto();
		await page.waitForTimeout(1000);
		await lessonPage.expectCardVisible();
		await lessonPage.showAnswer();
		await lessonPage.expectRatingButtonsVisible();
	});

	test("должен вернуться на главную при нажатии кнопки назад", async ({ page }) => {
		await lessonPage.goto();
		await lessonPage.goBack();
		await expect(page).toHaveURL("/home");
	});

	test("должен отобразить экран завершения после прохождения всех карточек", async ({ page }) => {
		await lessonPage.goto();
		await page.waitForTimeout(1000);

		const maxIterations = 50;
		let iteration = 0;

		while (iteration < maxIterations) {
			try {
				await lessonPage.showAnswer();
				await lessonPage.rateGood();
				await page.waitForTimeout(500);

				if (await lessonPage.completeTitle.isVisible({ timeout: 1000 })) {
					break;
				}
			} catch {
				break;
			}
			iteration++;
		}

		await lessonPage.expectCompleteScreen();
	});
});
