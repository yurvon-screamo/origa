import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { ExamplesPage, WordsPage } from "../../pages";

When('пользователь открывает страницу примеров', async ({ page }) => {
    const examplesPage = new ExamplesPage(page);
    await examplesPage.goto();
    await examplesPage.expectVisible();
});

When('нажимает навигацию к примерам', async ({ page }) => {
    // #823: the entry point is the words header button.
    const wordsPage = new WordsPage(page);
    await wordsPage.examplesButton.click({ timeout: 10_000 });
    await page.waitForURL(/\/examples$/, { timeout: 10_000 });
});

When('ищет пример {string} на странице примеров', async ({ page }, query: string) => {
    const examplesPage = new ExamplesPage(page);
    await examplesPage.searchInput.fill(query);
});

Then('страница примеров отображается', async ({ page }) => {
    const examplesPage = new ExamplesPage(page);
    await examplesPage.expectVisible(15_000);
});

Then('на странице примеров отображается пустое состояние', async ({ page }) => {
    const examplesPage = new ExamplesPage(page);
    await expect(examplesPage.emptyState).toBeVisible();
});

Then('отображается поле поиска примеров', async ({ page }) => {
    const examplesPage = new ExamplesPage(page);
    await expect(examplesPage.searchInput).toBeVisible();
});

Then('поле поиска примеров содержит {string}', async ({ page }, query: string) => {
    const examplesPage = new ExamplesPage(page);
    await expect(examplesPage.searchInput).toHaveValue(query, { timeout: 10_000 });
});
