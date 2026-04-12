import * as path from "path";
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

testWithFreshUser.describe("Words Page - Mark as Known", () => {
    testWithFreshUser("should display mark-as-known button on word card", async ({ page }) => {
        test.setTimeout(60_000);
        const wordsPage = await setupWordsPage(page);
        await addFirstWord(wordsPage);
        await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });

        const markKnownBtn = page.getByTestId("words-card-item").first().getByTestId("words-card-item-mark-known-btn");
        await expect(markKnownBtn).toBeVisible();
    });

    testWithFreshUser("should mark word as known and show in Learned filter", async ({ page }) => {
        test.setTimeout(60_000);
        const wordsPage = await setupWordsPage(page);
        await addFirstWord(wordsPage);
        await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });

        // Initially in "New" filter
        await wordsPage.selectFilter("Новые");
        expect(await wordsPage.getCardCount()).toBeGreaterThanOrEqual(1);

        // Mark as known
        await wordsPage.markCardAsKnownByIndex(0);
        await page.waitForTimeout(1000);

        // Now should appear in "Learned" filter
        await wordsPage.selectFilter("Изученные");
        await expect(wordsPage.emptyState).not.toBeVisible({ timeout: 5000 });
        expect(await wordsPage.getCardCount()).toBeGreaterThanOrEqual(1);
    });
});

testWithFreshUser.describe("Words Page - Anki Import", () => {
    testWithFreshUser("should display Anki tab in add words drawer", async ({ page }) => {
        test.setTimeout(60_000);
        const wordsPage = await setupWordsPage(page);

        await wordsPage.openAddModal();
        await expect(wordsPage.drawer).toBeVisible({ timeout: 5000 });

        await expect(wordsPage.ankiTab).toBeVisible();
    });

    testWithFreshUser("should switch to Anki tab and show drop zone", async ({ page }) => {
        test.setTimeout(60_000);
        const wordsPage = await setupWordsPage(page);

        await wordsPage.openAddModal();
        await expect(wordsPage.drawer).toBeVisible({ timeout: 5000 });

        await wordsPage.switchToAnkiTab();

        await expect(wordsPage.ankiDropZone).toBeVisible({ timeout: 5000 });
    });

    testWithFreshUser("should show error for invalid file", async ({ page }) => {
        test.setTimeout(60_000);
        const wordsPage = await setupWordsPage(page);

        await wordsPage.openAddModal();
        await expect(wordsPage.drawer).toBeVisible({ timeout: 5000 });

        await wordsPage.switchToAnkiTab();
        await expect(wordsPage.ankiDropZone).toBeVisible({ timeout: 5000 });

        await wordsPage.uploadAnkiFile("fixtures/sample.txt");

        await expect(wordsPage.ankiError).toBeVisible({ timeout: 10_000 });
    });
});

testWithFreshUser.describe("Words Page - OCR Image Recognition", () => {
    testWithFreshUser("should recognize Japanese text from image via OCR", async ({ page }) => {
        test.setTimeout(300_000);
        const wordsPage = await setupWordsPage(page);

        await wordsPage.openAddModal();
        await expect(wordsPage.drawer).toBeVisible({ timeout: 5000 });

        await wordsPage.switchToImageTab();

        await wordsPage.uploadImageFile(path.resolve(__dirname, "../../origa/src/ocr/ocr_example.jpg"));

        // Wait for OCR to complete and text analysis to finish
        // OCR downloads models (~50MB), then processes image, then auto-analyzes text
        await wordsPage.drawer.getByText(/Найдено/).waitFor({ state: "visible", timeout: 240_000 });

        // Verify key Japanese words were recognized from the test image
        const drawerText = await wordsPage.drawer.textContent({ timeout: 5000 });

        // The tokenizer produces base forms (kanji), not hiragana readings
        // Verify some of the key words from ocr_example.jpg appear as base forms
        const expectedWords = ["練習", "問題", "トイレ", "電車", "田中", "会議"];
        for (const word of expectedWords) {
            expect(drawerText).toContain(word);
        }
    });
});

testWithFreshUser.describe("Words Page - Audio Transcription", () => {
    testWithFreshUser("should transcribe Japanese audio and show word analysis", async ({ page }) => {
        test.setTimeout(300_000);
        const wordsPage = await setupWordsPage(page);

        await wordsPage.openAddModal();
        await expect(wordsPage.drawer).toBeVisible({ timeout: 5000 });

        await wordsPage.switchToAudioTab();

        await wordsPage.uploadAudioFile(path.resolve(__dirname, "../fixtures/standard_sample1.wav"));

        // Wait for Whisper model download + transcription + text analysis
        await wordsPage.drawer.getByText(/Найдено/).waitFor({ state: "visible", timeout: 240_000 });

        // Verify words were found
        const drawerText = await wordsPage.drawer.textContent({ timeout: 5000 });
        expect(drawerText).toContain("Найдено");
    });
});

testWithFreshUser.describe("Words Page - Pagination", () => {
    testWithFreshUser("should not show load-more button with few cards", async ({ page }) => {
        test.setTimeout(60_000);
        const wordsPage = await setupWordsPage(page);
        await addFirstWord(wordsPage);
        await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });

        // With only a few cards (< 50), load-more button should NOT be visible
        await expect(wordsPage.loadMoreButton).not.toBeVisible();
    });
});
