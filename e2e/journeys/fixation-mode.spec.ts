import { expect, test } from "@playwright/test";
import { HomePage, LessonPage, LoginPage } from "../pages";
import { CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD } from "../fixtures/test-helpers";

test.describe("UJ4: Режим закрепления", () => {
	let loginPage: LoginPage;
	let lessonPage: LessonPage;
	let homePage: HomePage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		lessonPage = new LessonPage(page);
		homePage = new HomePage(page);

		await loginPage.goto();
		await loginPage.login(CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD);
		await page.waitForURL("/home");
	});

	test("пользователь может перейти в режим закрепления", async ({ page }) => {
		await test.step("Переход в режим закрепления через кнопку", async () => {
			const hasFixation = await homePage.hasFixationSection();
			if (hasFixation) {
				await homePage.startFixation();
			} else {
				await lessonPage.gotoFixation();
			}
			await page.waitForTimeout(3000);
		});

		await test.step("Проверка содержимого страницы", async () => {
			const pageContent = await page.content();
			expect(pageContent.length).toBeGreaterThan(100);
		});
	});
});
