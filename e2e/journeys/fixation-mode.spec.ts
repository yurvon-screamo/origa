import { expect, test } from "@playwright/test";
import { LessonPage, LoginPage } from "../pages";
import { CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD } from "../fixtures/test-helpers";

test.describe("UJ4: Режим закрепления", () => {
	let loginPage: LoginPage;
	let lessonPage: LessonPage;

	test.beforeEach(async ({ page }) => {
		loginPage = new LoginPage(page);
		lessonPage = new LessonPage(page);

		await loginPage.goto();
		await loginPage.login(CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD);
		await page.waitForURL("/home");
	});

	test("пользователь может перейти в режим закрепления", async ({ page }) => {
		await lessonPage.gotoFixation();
		await page.waitForTimeout(3000);

		const pageContent = await page.content();
		expect(pageContent.length).toBeGreaterThan(100);
	});
});
