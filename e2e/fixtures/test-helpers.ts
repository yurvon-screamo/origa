import { expect, type Page } from "@playwright/test";
import { LoginPage } from "../pages/LoginPage";
import { KanjiPage } from "../pages/KanjiPage";
import { LessonPage } from "../pages/LessonPage";

const CONFIRMED_TEST_EMAIL = process.env.E2E_TEST_EMAIL!;
const CONFIRMED_TEST_PASSWORD = process.env.E2E_TEST_PASSWORD!;

export async function loginAsTestUser(page: Page): Promise<void> {
	const loginPage = new LoginPage(page);
	await loginPage.goto();
	await loginPage.login(CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD);
	await page.waitForURL("/home");
}

export async function addKanjiViaUI(page: Page, level: "N5" | "N4" | "N3" | "N2" | "N1", kanjies: string[]): Promise<void> {
	const kanjiPage = new KanjiPage(page);
	await kanjiPage.goto();
	await kanjiPage.expectVisible();
	
	await kanjiPage.clickAddButton();
	await kanjiPage.expectModalVisible();
	
	await kanjiPage.selectLevel(level);
	await page.waitForTimeout(1000);
	
	for (const kanji of kanjies) {
		await kanjiPage.selectKanji(kanji);
	}
	
	await kanjiPage.confirmAdd();
	await page.waitForTimeout(500);
	
	await kanjiPage.expectModalNotVisible();
}

export async function completeLessonWithRating(page: Page, rating: "again" | "hard" | "good" | "easy"): Promise<void> {
	const lessonPage = new LessonPage(page);
	await lessonPage.goto();
	await page.waitForTimeout(1000);
	
	const maxIterations = 100;
	let iteration = 0;
	
	while (iteration < maxIterations) {
		const hasCard = await lessonPage.showAnswerButton.isVisible({ timeout: 2000 }).catch(() => false);
		
		if (!hasCard) {
			const hasComplete = await lessonPage.completeTitle.isVisible({ timeout: 1000 }).catch(() => false);
			if (hasComplete) break;
			
			const hasEmpty = await lessonPage.emptyStateText.isVisible({ timeout: 1000 }).catch(() => false);
			if (hasEmpty) break;
			
			await page.waitForTimeout(500);
			iteration++;
			continue;
		}
		
		await lessonPage.showAnswer();
		await page.waitForTimeout(200);
		
		switch (rating) {
			case "again":
				await lessonPage.rateAgain();
				break;
			case "hard":
				await lessonPage.rateHard();
				break;
			case "good":
				await lessonPage.rateGood();
				break;
			case "easy":
				await lessonPage.rateEasy();
				break;
		}
		
		await page.waitForTimeout(300);
		iteration++;
	}
}

export async function completeAllLessonCards(page: Page): Promise<void> {
	const lessonPage = new LessonPage(page);
	await lessonPage.goto();
	await page.waitForTimeout(1000);
	
	const maxIterations = 100;
	let iteration = 0;
	
	while (iteration < maxIterations) {
		const hasCard = await lessonPage.showAnswerButton.isVisible({ timeout: 2000 }).catch(() => false);
		
		if (!hasCard) {
			const hasComplete = await lessonPage.completeTitle.isVisible({ timeout: 1000 }).catch(() => false);
			if (hasComplete) break;
			
			const hasEmpty = await lessonPage.emptyStateText.isVisible({ timeout: 1000 }).catch(() => false);
			if (hasEmpty) break;
			
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
}

export async function expectLessonCompleted(page: Page): Promise<void> {
	const lessonPage = new LessonPage(page);
	await expect(lessonPage.completeTitle).toBeVisible({ timeout: 5000 });
}

export async function expectLessonEmpty(page: Page): Promise<void> {
	const lessonPage = new LessonPage(page);
	await expect(lessonPage.emptyStateText).toBeVisible({ timeout: 5000 });
}

export { CONFIRMED_TEST_EMAIL, CONFIRMED_TEST_PASSWORD };
