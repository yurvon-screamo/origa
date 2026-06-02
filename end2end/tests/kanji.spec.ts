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
    const firstKanji = kanjiPage.drawer.getByTestId("kanji-drawer-item").first();
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
        const firstN5 = kanjiPage.drawer.getByTestId("kanji-drawer-item").first();
        await expect(firstN5).toBeVisible({ timeout: 10_000 });
        await firstN5.click();
        await kanjiPage.addSelectedKanji();

        await kanjiPage.openAddModal();
        await kanjiPage.selectLevel("N4");
        const firstN4 = kanjiPage.drawer.getByTestId("kanji-drawer-item").first();
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
        const items = kanjiPage.drawer.getByTestId("kanji-drawer-item");
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

        const firstKanjiChar = await kanjiPage.kanjiGrid.locator(".kanji-card-kanji-char").first().textContent();
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
        await setupKanjiPage(page);
        await page.getByTestId("sidebar-tab-home").click();
        await page.waitForURL(/\/home$/, { timeout: 10000 });
        await expect(page).toHaveURL(/\/home$/);
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

testWithFreshUser.describe("Kanji Page - Detail Page", () => {
    testWithFreshUser("should navigate to detail page when clicking kanji card", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        const kanjiLink = page.locator('a[href*="/kanji/"]').first();
        await kanjiLink.click();

        await page.waitForURL(/\/kanji\//, { timeout: 5_000 });
        const breadcrumbs = page.locator(".kanji-breadcrumbs");
        await expect(breadcrumbs).toBeVisible({ timeout: 5_000 });
    });

    testWithFreshUser("should display content on detail page", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        const kanjiLink = page.locator('a[href*="/kanji/"]').first();
        await kanjiLink.click();

        await page.waitForURL(/\/kanji\//, { timeout: 5_000 });
        const breadcrumbs = page.locator(".kanji-breadcrumbs");
        await expect(breadcrumbs).toBeVisible({ timeout: 5_000 });

        const heroCard = page.locator(".kanji-detail-hero-card").first();
        await expect(heroCard).toBeVisible({ timeout: 5_000 });
    });

    testWithFreshUser("should navigate back to kanji list via back button", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        const kanjiLink = page.locator('a[href*="/kanji/"]').first();
        await kanjiLink.click();

        await page.waitForURL(/\/kanji\//, { timeout: 5_000 });

        await page.goBack();
        await page.waitForURL(/\/kanji$/, { timeout: 5_000 });
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 5_000 });
    });
});

testWithFreshUser.describe("Kanji Page - Mark as Known", () => {
    testWithFreshUser("should display mark-as-known button on kanji card", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        const markKnownBtn = page.getByTestId("kanji-card-item-mark-known-btn").first();
        await expect(markKnownBtn).toBeVisible();
    });

    testWithFreshUser("should mark kanji as known and show in Learned filter", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        await kanjiPage.selectFilter("Новые");
        expect(await kanjiPage.getCardCount()).toBeGreaterThanOrEqual(1);

        await kanjiPage.markCardAsKnownByIndex(0);

        await kanjiPage.selectFilter("Изученные");
        await expect(kanjiPage.emptyState).not.toBeVisible({ timeout: 5000 });
        expect(await kanjiPage.getCardCount()).toBeGreaterThanOrEqual(1);
    });

    testWithFreshUser("should hide mark-as-known button for already learned kanji", async ({ page }) => {
        test.setTimeout(60_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        // Pre-condition: mark-as-known button must exist before we test hiding it
        const markKnownBtn = page.getByTestId("kanji-card-item-mark-known-btn").first();
        await expect(markKnownBtn).toBeVisible();

        // Click mark-as-known
        await kanjiPage.markCardAsKnownByIndex(0);

        // Wait for card to re-render with learned status
        await page.waitForTimeout(1000);

        // After marking as known, the button should be hidden
        await expect(markKnownBtn).not.toBeVisible();
    });
});

testWithFreshUser.describe("Kanji Page - Favorite Instant UI Update", () => {
    testWithFreshUser("should update favorite icon immediately after toggle", async ({ page }) => {
        test.setTimeout(90_000);
        const kanjiPage = await setupKanjiPage(page);
        await addFirstKanji(kanjiPage);
        await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 10_000 });

        await expect.poll(async () => await kanjiPage.isFavorited(0), { timeout: 5000 }).toBe(false);

        // Toggle to favorite — strict 1000ms timeout verifies optimistic UI update
        const card = page.getByTestId("kanji-card-item").nth(0);
        const btn = card.getByTestId("kanji-card-item-favorite-btn");
        await btn.dispatchEvent("click");
        await expect.poll(async () => await kanjiPage.isFavorited(0), { timeout: 1000 }).toBe(true);

        // Wait for background reload and verify persistence
        await page.waitForTimeout(2000);
        await expect.poll(async () => await kanjiPage.isFavorited(0), { timeout: 1000 }).toBe(true);

        // Toggle back to unfavorite
        await btn.dispatchEvent("click");
        await expect.poll(async () => await kanjiPage.isFavorited(0), { timeout: 1000 }).toBe(false);

        // Verify persistence after reload
        await page.waitForTimeout(2000);
        await expect.poll(async () => await kanjiPage.isFavorited(0), { timeout: 1000 }).toBe(false);
    });
});
