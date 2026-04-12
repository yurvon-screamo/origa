import { test, expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { GrammarPage } from "../pages";

async function setupGrammarPage(page: Page): Promise<GrammarPage> {
    await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10000 });
    await page.getByTestId("onboarding-skip").click();
    await page.waitForURL(/\/home$/, { timeout: 10000 });

    const grammarPage = new GrammarPage(page);
    await grammarPage.goto();
    await grammarPage.expectGrammarVisible();
    return grammarPage;
}

testWithFreshUser.describe("Grammar Page - CRUD", () => {
    testWithFreshUser("should display empty state for new user", async ({ page }) => {
        const grammarPage = await setupGrammarPage(page);
        await expect(grammarPage.emptyState).toBeVisible();
    });

    testWithFreshUser("should add N5 grammar card", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();

        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });
        await expect(grammarPage.emptyState).not.toBeVisible();
    });

    testWithFreshUser("should add grammar cards for multiple levels", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();

        await grammarPage.openAddModal();
        await grammarPage.selectLevel("N4");
        const n4Rule = grammarPage.drawer.locator(".border.cursor-pointer").first();
        if (await n4Rule.isVisible().catch(() => false)) {
            await n4Rule.click();
            await grammarPage.addSelectedRules();
        }

        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });
        expect(await grammarPage.getCardCount()).toBeGreaterThanOrEqual(1);
    });

    testWithFreshUser("should select all rules and add them", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectAllRules();
        await expect(grammarPage.drawer.getByText(/Выбрано: \d+ правил/)).toBeVisible();
        await grammarPage.addSelectedRules();

        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });
        expect(await grammarPage.getCardCount()).toBeGreaterThan(0);
    });

    testWithFreshUser("should delete a grammar card", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        const countBefore = await grammarPage.getCardCount();
        expect(countBefore).toBeGreaterThan(0);

        await grammarPage.deleteCardByIndex(0);
        // Wait for UI to update after deletion
        await page.waitForTimeout(500);
        expect(await grammarPage.getCardCount()).toBe(countBefore - 1);
    });

    testWithFreshUser("should cancel card deletion", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        const countBefore = await grammarPage.getCardCount();
        await grammarPage.cancelDeleteCardByIndex(0);
        expect(await grammarPage.getCardCount()).toBe(countBefore);
    });
});

testWithFreshUser.describe("Grammar Page - Search & Filters", () => {
    testWithFreshUser("should search grammar cards", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.selectRule("～ません");
        await grammarPage.selectRule("～ました");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        await grammarPage.searchGrammar("ます");
        await expect(grammarPage.emptyState).not.toBeVisible();
        expect(await grammarPage.getCardCount()).toBeGreaterThan(0);

        await grammarPage.searchGrammar("xyznonexistent");
        await expect(grammarPage.emptyState).toBeVisible({ timeout: 5000 });

        await grammarPage.searchGrammar("");
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 5000 });
    });

    testWithFreshUser("should filter cards by status", async ({ page }) => {
        test.setTimeout(90_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        await grammarPage.selectFilter("Все");
        expect(await grammarPage.getCardCount()).toBeGreaterThanOrEqual(1);

        await grammarPage.selectFilter("Новые");
        expect(await grammarPage.getCardCount()).toBeGreaterThanOrEqual(1);

        await grammarPage.selectFilter("Изученные");
        await expect(grammarPage.emptyState).toBeVisible({ timeout: 5000 });
    });

    testWithFreshUser("should navigate back to home", async ({ page }) => {
        const grammarPage = await setupGrammarPage(page);
        await grammarPage.clickBack();
        await page.waitForURL(/\/home$/, { timeout: 10000 });
        await expect(page).toHaveURL(/\/home$/);
    });
});

testWithFreshUser.describe("Grammar Page - Mark as Known", () => {
    testWithFreshUser("should display mark-as-known button on grammar card", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);
        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        const markKnownBtn = page.getByTestId("grammar-card-item").first().getByTestId("grammar-card-item-mark-known-btn");
        await expect(markKnownBtn).toBeVisible();
    });

    testWithFreshUser("should mark grammar as known and show in Learned filter", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);
        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        await grammarPage.markCardAsKnownByIndex(0);

        await grammarPage.selectFilter("Изученные");
        await expect(grammarPage.emptyState).not.toBeVisible({ timeout: 5000 });
        expect(await grammarPage.getCardCount()).toBeGreaterThanOrEqual(1);
    });
});

testWithFreshUser.describe("Grammar Page - Pagination", () => {
    testWithFreshUser("should not show load-more button with few cards", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        // With only 1 card (< 50), load-more button should NOT be visible
        await expect(grammarPage.loadMoreButton).not.toBeVisible();
    });

    testWithFreshUser("should show load-more button when many grammar rules exist", async ({ page }) => {
        test.setTimeout(90_000);
        const grammarPage = await setupGrammarPage(page);

        // Add all N5 grammar rules
        await grammarPage.openAddModal();
        await grammarPage.selectAllRules();
        await grammarPage.addSelectedRules();

        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });
        const totalCount = await grammarPage.getCardCount();

        // If enough rules to trigger pagination
        if (totalCount >= 50) {
            await expect(grammarPage.loadMoreButton).toBeVisible({ timeout: 5000 });
            expect(totalCount).toBe(50);
        } else {
            // Not enough rules for pagination - button should not be visible
            await expect(grammarPage.loadMoreButton).not.toBeVisible();
        }
    });

    testWithFreshUser("should reset pagination when changing filter with many cards", async ({ page }) => {
        test.setTimeout(90_000);
        const grammarPage = await setupGrammarPage(page);

        // Add all N5 rules first
        await grammarPage.openAddModal();
        await grammarPage.selectAllRules();
        await grammarPage.addSelectedRules();

        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        // Also add N4 rules to get more cards
        await grammarPage.openAddModal();
        await grammarPage.selectLevel("N4");
        await grammarPage.selectAllRules();
        await grammarPage.addSelectedRules();

        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        // Only test if pagination is active
        if (await grammarPage.isLoadMoreVisible().catch(() => false)) {
            // Click load more to expand
            await grammarPage.clickLoadMore();
            await page.waitForTimeout(500);
            const expandedCount = await grammarPage.getCardCount();
            expect(expandedCount).toBeGreaterThan(50);

            // Change filter - should reset visible cards
            await grammarPage.selectFilter("Новые");
            const resetCount = await grammarPage.getCardCount();
            expect(resetCount).toBeLessThanOrEqual(50);
        }
    });
});
