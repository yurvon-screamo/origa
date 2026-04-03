import { test, expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { WordsPage, SetsPage } from "../pages";

async function setupWordsPage(page: Page): Promise<WordsPage> {
    await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10000 });
    await page.getByTestId("onboarding-skip").click();
    await page.waitForURL(/\/home$/, { timeout: 10000 });

    const wordsPage = new WordsPage(page);
    await wordsPage.goto();
    await wordsPage.expectWordsVisible();
    return wordsPage;
}

async function addFirstWord(wordsPage: WordsPage): Promise<void> {
    await wordsPage.openAddModal();
    await wordsPage.enterText("私は本を読みます");
    await wordsPage.analyzeText();
    await wordsPage.selectFirstWord();
    await wordsPage.addSelectedWords();
}

testWithFreshUser.describe("Words Page - CRUD", () => {
    testWithFreshUser("should display empty state for new user", async ({ page }) => {
        const wordsPage = await setupWordsPage(page);
        await expect(wordsPage.emptyState).toBeVisible();
    });

    testWithFreshUser("should add word via text analysis", async ({ page }) => {
        test.setTimeout(60_000);
        const wordsPage = await setupWordsPage(page);
        await addFirstWord(wordsPage);

        await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });
        await expect(wordsPage.emptyState).not.toBeVisible();
    });

    testWithFreshUser("should cancel adding words", async ({ page }) => {
        test.setTimeout(60_000);
        const wordsPage = await setupWordsPage(page);

        await wordsPage.openAddModal();
        await wordsPage.enterText("私は本を読みます");
        await wordsPage.analyzeText();
        await wordsPage.cancelAddModal();

        await expect(wordsPage.drawer).not.toBeVisible();
        await expect(wordsPage.emptyState).toBeVisible();
    });

    testWithFreshUser("should delete a word card", async ({ page }) => {
        test.setTimeout(60_000);
        const wordsPage = await setupWordsPage(page);
        await addFirstWord(wordsPage);
        await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });

        const countBefore = await wordsPage.getCardCount();
        expect(countBefore).toBeGreaterThan(0);

        await wordsPage.deleteCardByIndex(0);
        await page.waitForTimeout(500);
        expect(await wordsPage.getCardCount()).toBe(countBefore - 1);
    });

    testWithFreshUser("should cancel card deletion", async ({ page }) => {
        test.setTimeout(60_000);
        const wordsPage = await setupWordsPage(page);
        await addFirstWord(wordsPage);
        await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });

        const countBefore = await wordsPage.getCardCount();
        await wordsPage.cancelDeleteCardByIndex(0);
        expect(await wordsPage.getCardCount()).toBe(countBefore);
    });
});

testWithFreshUser.describe("Words Page - Search & Filters", () => {
    testWithFreshUser("should search word cards", async ({ page }) => {
        test.setTimeout(60_000);
        const wordsPage = await setupWordsPage(page);
        await addFirstWord(wordsPage);
        await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });

        await expect(page.getByTestId("words-card-item").first()).toBeVisible({ timeout: 10_000 });
        const firstWordText = await page
            .getByTestId("words-card-item")
            .first()
            .locator("ruby")
            .first()
            .evaluate((el) => {
                let text = '';
                for (const node of Array.from(el.childNodes)) {
                    if (node.nodeType === Node.TEXT_NODE) {
                        text += node.textContent;
                    }
                }
                return text;
            });
        if (firstWordText) {
            await wordsPage.searchWords(firstWordText.trim());
            await expect(wordsPage.emptyState).not.toBeVisible();
        }

        await wordsPage.searchWords("xyznonexistent");
        await expect(wordsPage.emptyState).toBeVisible({ timeout: 5000 });

        await wordsPage.searchWords("");
        await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 5000 });
    });

    testWithFreshUser("should filter cards by status", async ({ page }) => {
        test.setTimeout(60_000);
        const wordsPage = await setupWordsPage(page);
        await addFirstWord(wordsPage);
        await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });

        await wordsPage.selectFilter("Все");
        expect(await wordsPage.getCardCount()).toBeGreaterThanOrEqual(1);

        await wordsPage.selectFilter("Новые");
        expect(await wordsPage.getCardCount()).toBeGreaterThanOrEqual(1);

        await wordsPage.selectFilter("Изученные");
        await expect(wordsPage.emptyState).toBeVisible({ timeout: 5000 });
    });
});

testWithFreshUser.describe("Words Page - Navigation", () => {
    testWithFreshUser("should navigate back to home", async ({ page }) => {
        const wordsPage = await setupWordsPage(page);
        await wordsPage.clickBack();
        await page.waitForURL(/\/home$/, { timeout: 10000 });
        await expect(page).toHaveURL(/\/home$/);
    });

    testWithFreshUser("should navigate to sets page", async ({ page }) => {
        const wordsPage = await setupWordsPage(page);
        await wordsPage.clickSets();
        await page.waitForURL(/\/sets$/, { timeout: 10000 });
        await expect(page).toHaveURL(/\/sets$/);

        const setsPage = new SetsPage(page);
        await setsPage.expectSetsVisible();
    });
});
