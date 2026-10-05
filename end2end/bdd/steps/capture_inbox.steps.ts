import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { WordsPage } from "../../pages";

// Zero-tap intake (slice IN-1): payloads arrive through the app's e2e seam
// and must land the user directly on the word preview or a actionable
// fallback — never back on the source tabs.

When('пользователю поделились текстом {string}', async ({ page }, text: string) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.shareText(text);
});

When('пользователю поделились файлом {string} типа {string}', async ({ page }, fileName: string, mime: string) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.shareFile(fileName, mime);
});

Then('drawer добавления открыт сразу на превью слов', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.expectInboxPreview();
});

Then('вкладки источников скрыты', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.expectInboxTabsHidden();
});

Then('вкладки источников отображаются', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.expectInboxTabsVisible();
});

Then('открыт drawer добавления', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await expect(wordsPage.drawer).toBeVisible({ timeout: 10_000 });
});

Then('отображается ошибка обработки контента', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.expectInboxError();
});

When('пользователь возвращается к ручному вводу', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.returnToManualInput();
});
