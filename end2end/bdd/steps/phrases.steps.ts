import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { PhrasesPage } from "../../pages";

When('пользователь открывает страницу фраз', async ({ page }) => {
    const phrasesPage = new PhrasesPage(page);
    await phrasesPage.goto();
    await phrasesPage.expectPhrasesVisible();
});

Then('на странице фраз отображается пустое состояние', async ({ page }) => {
    const phrasesPage = new PhrasesPage(page);
    await expect(phrasesPage.emptyState).toBeVisible();
});

Then('отображается поле поиска фраз', async ({ page }) => {
    const phrasesPage = new PhrasesPage(page);
    await expect(phrasesPage.searchInput).toBeVisible();
});

Then('отображается вкладка фраз в навигации', async ({ page }) => {
    // Viewport is set up by the 'пользователь устанавливает мобильный размер
    // экрана' Given step — the bottom-tab nav is mobile-only (lg:hidden).
    await expect(page.getByTestId("bottom-tab")).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId("bottom-tab-tab-phrases")).toBeVisible({ timeout: 10_000 });
});

When('нажимает кнопку возврата с фраз', async ({ page }) => {
    await page.goto("/home");
    await page.waitForURL(/\/home$/, { timeout: 10_000 });
});

Then('отображаются кнопки фильтрации фраз', async ({ page }) => {
    await expect(page.getByTestId(/phrase.*filter|filter.*phrase/).first()).toBeVisible({ timeout: 10_000 });
});

Then('карточки фраз имеют непустой текст', async ({ page }) => {
    await expect(page.getByTestId("phrases-card-item").first()).toBeVisible({ timeout: 30_000 });
    const text = await page.getByTestId("phrases-card-item").first().textContent();
    expect(text?.trim().length ?? 0).toBeGreaterThan(0);
});

Then('текст фраз содержит фуригану', async ({ page }) => {
    // Furigana arrives either from the precompute blob (#521 fast path)
    // or from the background tokenizer warmup — generous timeout covers
    // the latter in CI (~344 MB dictionary download from the mirror).
    // Anchored to the card (not the translator span): the translator's
    // no-language fallback branch renders plain text without inner
    // test ids, and the card is the stable container either way.
    const first = page.getByTestId("phrases-card-item").first();
    await expect(first).toBeVisible({ timeout: 30_000 });
    await expect(
        first.locator(".furigana-ruby").first(),
        "phrase text must render kanji with furigana ruby",
    ).toBeVisible({ timeout: 120_000 });
});

When('ищет фразы {string}', async ({ page }, query: string) => {
    const phrasesPage = new PhrasesPage(page);
    await phrasesPage.searchPhrases(query);
    await page.waitForTimeout(500);
});

Then('на странице фраз нет карточек', async ({ page }) => {
    await expect(page.getByTestId("phrases-card-item")).toHaveCount(0, { timeout: 10_000 });
});

/**
 * Network log across the phrases-page-load When-step of the lazy-load
 * scenario (#540-В1): the Then-step counts phrase data chunk requests.
 */
let phrasesRequestLog: string[] = [];

When('пользователь открывает страницу фраз с записью сетевых запросов', async ({ page }) => {
    phrasesRequestLog = [];
    page.on("request", (request) => phrasesRequestLog.push(request.url()));
    const phrasesPage = new PhrasesPage(page);
    await phrasesPage.goto();
    await phrasesPage.expectPhrasesVisible();
});

Then('количество загруженных чанков данных фраз ограничено видимой пачкой', async ({ page }) => {
    const chunkUrl = /\/phrases\/data\/p\d{4}\.json/;
    const countNow = () => phrasesRequestLog.filter((url) => chunkUrl.test(url)).length;

    // Wait for the request stream to stabilize: two consecutive samples
    // with the same count mean the burst is over. The pre-#540 behavior
    // (fetch EVERY chunk of the user's ~6k phrases) kept the stream busy
    // for a long time and would sail far past the bound below; the lazy
    // visible-slice load settles quickly on a few dozen chunks.
    let previous = -1;
    for (let i = 0; i < 20; i++) {
        await page.waitForTimeout(500);
        const current = countNow();
        if (current === previous) break;
        previous = current;
    }

    const finalCount = countNow();
    expect(
        finalCount,
        `the lazy phrases page must load only the visible slice's chunks (#540), got ${finalCount}`,
    ).toBeLessThanOrEqual(60);
    // Positive control: data WAS loaded (the preceding step already
    // asserted non-empty card text, this guards against a vacuous zero).
    expect(finalCount, "the visible slice must actually fetch its chunks").toBeGreaterThan(0);
});

When('удаляет первую фразу', async ({ page }) => {
    await page.getByTestId("phrases-card-item").first().locator('[data-testid*="delete"]').first().click();
});

Then('отображается сообщение о подтверждении удаления', async ({ page }) => {
    const phrasesPage = new PhrasesPage(page);
    await expect(phrasesPage.deleteModal).toBeVisible();
});
