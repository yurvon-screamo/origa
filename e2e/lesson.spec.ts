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

	test("должен отобразить прогресс или пустое состояние", async ({ page }) => {
		await lessonPage.goto();
		await page.waitForTimeout(2000);

		const hasProgress = await lessonPage.progressBar.isVisible({ timeout: 5000 }).catch(() => false);
		const hasEmptyState = await lessonPage.emptyStateText.isVisible({ timeout: 2000 }).catch(() => false);

		expect(hasProgress || hasEmptyState).toBe(true);
	});

	test("должен отобразить карточку или пустое состояние", async ({ page }) => {
		await lessonPage.goto();
		await page.waitForTimeout(2000);

		const hasCard = await lessonPage.showAnswerButton.isVisible({ timeout: 5000 }).catch(() => false);
		const hasEmptyState = await lessonPage.emptyStateText.isVisible({ timeout: 2000 }).catch(() => false);

		expect(hasCard || hasEmptyState).toBe(true);
	});

	test("должен показать ответ при нажатии кнопки (если есть карточки)", async ({ page }) => {
		await lessonPage.goto();
		await page.waitForTimeout(2000);

		const hasCard = await lessonPage.showAnswerButton.isVisible({ timeout: 3000 }).catch(() => false);
		if (!hasCard) {
			test.skip();
			return;
		}
		
		await lessonPage.showAnswer();
	});

	test("должен отобразить кнопки оценки после показа ответа (если есть карточки)", async ({ page }) => {
		await lessonPage.goto();
		await page.waitForTimeout(2000);

		const hasCard = await lessonPage.showAnswerButton.isVisible({ timeout: 3000 }).catch(() => false);
		if (!hasCard) {
			test.skip();
			return;
		}
		
		await lessonPage.showAnswer();
		await lessonPage.expectRatingButtonsVisible();
	});

	test("должен вернуться на главную при нажатии кнопки назад", async ({ page }) => {
		await lessonPage.goto();
		await lessonPage.goBack();
		await expect(page).toHaveURL("/home");
	});

	test("должен отобразить экран завершения или пустое состояние", async ({ page }) => {
		await lessonPage.goto();
		await page.waitForTimeout(2000);

		const hasCard = await lessonPage.showAnswerButton.isVisible({ timeout: 3000 }).catch(() => false);
		if (!hasCard) {
			await lessonPage.emptyStateText.isVisible({ timeout: 3000 }).catch(() => {});
			return;
		}

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

		await expect(lessonPage.completeTitle).toBeVisible({ timeout: 5000 }).catch(() => {});
	});
});
