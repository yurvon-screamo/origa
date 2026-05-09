import { test, expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { skipOnboarding } from "../helpers/navigation";
import { KanjiPage } from "../pages";

async function setupKanjiPage(page: Page): Promise<KanjiPage> {
    await skipOnboarding(page);

    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await kanjiPage.expectKanjiVisible();
    return kanjiPage;
}

async function addFirstKanji(kanjiPage: KanjiPage): Promise<void> {
    await kanjiPage.openAddModal();
    const firstKanji = kanjiPage.drawer.locator(".border.cursor-pointer").first();
    await expect(firstKanji).toBeVisible({ timeout: 10_000 });
    await firstKanji.click();
    await kanjiPage.addSelectedKanji();
}

testWithFreshUser.describe("Kanji Page - CRUD", () => {
    testWithFreshUser("should display empty state for new user", async ({ page }) => {
        const kanjiPage = await setupKanjiPage(page);
        await expect(kanjiPage.emptyState).toBeVisible();
    });

    testWithFreshUser("should add N5 kanji card", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);

        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });
        await expect(kanjiPage.emptyState).not.toBeVisible();
    });

    testWithFreshUser("should add kanji from multiple levels", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);

        await kanjiPage.openAddModal();
        const firstN5 = kanjiPage.drawer.locator(".border.cursor-pointer").first();
        await expect(firstN5).toBeVisible({ timeout: 10_000 });
        await firstN5.click();
        await kanjiPage.addSelectedKanji();

        await kanjiPage.openAddModal();
        await kanjiPage.selectLevel("N4");
        const firstN4 = kanjiPage.drawer.locator(".border.cursor-pointer").first();
        if (await firstN4.isVisible().catch(() => false)) {
            await firstN4.click();
            await kanjiPage.addSelectedKanji();
        }

        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });
        expect(await kanjiPage.getCardCount()).toBeGreaterThanOrEqual(1);
    });

    testWithFreshUser("should select all kanji and add them", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);

        await kanjiPage.openAddModal();
        await kanjiPage.selectAllKanji();
        await kanjiPage.addSelectedKanji();

        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });
        expect(await kanjiPage.getCardCount()).toBeGreaterThan(0);
    });

    testWithFreshUser("should delete a kanji card", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        const countBefore = await kanjiPage.getCardCount();
        expect(countBefore).toBeGreaterThan(0);

        await kanjiPage.deleteCardByIndex(0);
        await expect.poll(() => kanjiPage.getCardCount()).toBe(countBefore - 1);
    });

    testWithFreshUser("should cancel card deletion", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        const countBefore = await kanjiPage.getCardCount();
        await kanjiPage.cancelDeleteCardByIndex(0);
        expect(await kanjiPage.getCardCount()).toBe(countBefore);
    });
});

testWithFreshUser.describe("Kanji Page - Search & Filters", () => {
    testWithFreshUser("should search kanji cards", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);

        await kanjiPage.openAddModal();
        const items = kanjiPage.drawer.locator(".border.cursor-pointer");
        const first = items.first();
        const second = items.nth(1);
        await expect(first).toBeVisible({ timeout: 10_000 });
        if (await second.isVisible().catch(() => false)) {
            await first.click();
            await second.click();
        } else {
            await first.click();
        }
        await kanjiPage.addSelectedKanji();
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        const firstKanjiChar = await kanjiPage.kanjiGrid.locator(".card").first().locator(".font-serif").first().textContent();
        if (firstKanjiChar) {
            await kanjiPage.searchKanji(firstKanjiChar.trim());
            await expect(kanjiPage.emptyState).not.toBeVisible();
        }

        await kanjiPage.searchKanji("xyznonexistent");
        await expect(kanjiPage.emptyState).toBeVisible({ timeout: 5000 });

        await kanjiPage.searchKanji("");
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 5000 });
    });

    testWithFreshUser("should filter cards by status", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        await kanjiPage.selectFilter("Все");
        expect(await kanjiPage.getCardCount()).toBe(1);

        await kanjiPage.selectFilter("Новые");
        expect(await kanjiPage.getCardCount()).toBe(1);

        await kanjiPage.selectFilter("Изученные");
        await expect(kanjiPage.emptyState).toBeVisible({ timeout: 5000 });
    });
});

testWithFreshUser.describe("Kanji Page - Navigation", () => {
    testWithFreshUser("should navigate to home via sidebar", async ({ page }) => {
        const kanjiPage = await setupKanjiPage(page);
        await page.getByTestId("sidebar-tab-home").click();
        await page.waitForURL(/\/home$/, { timeout: 10000 });
        await expect(page).toHaveURL(/\/home$/);
    });
});

testWithFreshUser.describe("Kanji Page - Mark as Known", () => {
    testWithFreshUser("should display mark-as-known button on kanji card", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        const markKnownBtn = page.getByTestId("kanji-card-item").first().getByTestId("kanji-card-item-mark-known-btn");
        await expect(markKnownBtn).toBeVisible();
    });

    testWithFreshUser("should mark kanji as known and show in Learned filter", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        const markKnownBtn = page.getByTestId("kanji-card-item").first().getByTestId("kanji-card-item-mark-known-btn");
        await markKnownBtn.click();

        await kanjiPage.selectFilter("Изученные");
        await expect(kanjiPage.emptyState).not.toBeVisible({ timeout: 5000 });
        expect(await kanjiPage.getCardCount()).toBeGreaterThanOrEqual(1);
    });
});

testWithFreshUser.describe("Kanji Page - Pagination", () => {
    testWithFreshUser("should not show load-more button with few cards", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        // With only 1 card (< 50), load-more button should NOT be visible
        await expect(kanjiPage.loadMoreButton).not.toBeVisible();
    });

    testWithFreshUser("should show load-more button when many kanji exist", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);

        // Add all N5 kanji (~80 kanji, well above the 50 threshold)
        await kanjiPage.openAddModal();
        await kanjiPage.selectAllKanji();
        await kanjiPage.addSelectedKanji();

        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        // Verify load-more button appears
        await expect(kanjiPage.loadMoreButton).toBeVisible({ timeout: 5000 });

        // Verify only 50 cards are rendered initially
        const visibleCount = await kanjiPage.getCardCount();
        expect(visibleCount).toBe(50);
    });

    testWithFreshUser("should load more cards on clicking load-more button", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);

        await kanjiPage.openAddModal();
        await kanjiPage.selectAllKanji();
        await kanjiPage.addSelectedKanji();

        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });
        await expect(kanjiPage.loadMoreButton).toBeVisible({ timeout: 5000 });

        const initialCount = await kanjiPage.getCardCount();
        expect(initialCount).toBe(50);

        // Click load more
        await kanjiPage.clickLoadMore();
        await expect(page.getByTestId("kanji-card-item").nth(initialCount)).toBeVisible({ timeout: 5000 });

        // More cards should be visible now
        const newCount = await kanjiPage.getCardCount();
        expect(newCount).toBeGreaterThan(initialCount);
    });

    testWithFreshUser("should reset pagination when changing filter", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);

        await kanjiPage.openAddModal();
        await kanjiPage.selectAllKanji();
        await kanjiPage.addSelectedKanji();

        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });
        await expect(kanjiPage.loadMoreButton).toBeVisible({ timeout: 5000 });

        // Click load more to expand
        await kanjiPage.clickLoadMore();
        await expect(page.getByTestId("kanji-card-item").nth(50)).toBeVisible({ timeout: 5000 });
        const expandedCount = await kanjiPage.getCardCount();
        expect(expandedCount).toBeGreaterThan(50);

        // Change filter - should reset visible cards to 50
        await kanjiPage.selectFilter("Новые");
        const resetCount = await kanjiPage.getCardCount();
        expect(resetCount).toBeLessThanOrEqual(50);
    });
});

testWithFreshUser.describe("Kanji Page - Detail Drawer", () => {
    testWithFreshUser("should open detail drawer when clicking kanji card", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        const firstCard = page.getByTestId("kanji-card-item").first();
        await firstCard.click();

        const detailDrawer = page.getByTestId("kanji-detail-drawer");
        await expect(detailDrawer).toBeVisible({ timeout: 5_000 });
    });

    testWithFreshUser("should display content in detail drawer", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        const firstCard = page.getByTestId("kanji-card-item").first();
        await firstCard.click();

        const detailDrawer = page.getByTestId("kanji-detail-drawer");
        await expect(detailDrawer).toBeVisible({ timeout: 5_000 });

        // Drawer should contain some text content (kanji details)
        const drawerText = await detailDrawer.textContent({ timeout: 5_000 });
        expect(drawerText).toBeTruthy();
        expect(drawerText!.length).toBeGreaterThan(0);
    });

    testWithFreshUser("should close detail drawer via close button", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        const firstCard = page.getByTestId("kanji-card-item").first();
        await firstCard.click();

        const detailDrawer = page.getByTestId("kanji-detail-drawer");
        await expect(detailDrawer).toBeVisible({ timeout: 5_000 });

        await kanjiPage.closeDetailDrawer();
        await expect(detailDrawer).not.toBeVisible({ timeout: 5_000 });
    });
});
