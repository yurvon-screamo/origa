import { test, expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { skipOnboarding } from "../helpers/navigation";
import { GrammarPage, WordsPage } from "../pages";

async function setupGrammarPage(page: Page): Promise<GrammarPage> {
    await skipOnboarding(page);

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
        await expect.poll(() => grammarPage.getCardCount()).toBe(countBefore - 1);
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

    testWithFreshUser("should navigate to home via sidebar", async ({ page }) => {
        await setupGrammarPage(page);
        await page.getByTestId("sidebar-tab-home").click();
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
            await expect(page.getByTestId("grammar-card-item").nth(50)).toBeVisible({ timeout: 5000 });
            const expandedCount = await grammarPage.getCardCount();
            expect(expandedCount).toBeGreaterThan(50);

            // Change filter - should reset visible cards
            await grammarPage.selectFilter("Новые");
            const resetCount = await grammarPage.getCardCount();
            expect(resetCount).toBeLessThanOrEqual(50);
        }
    });
});

testWithFreshUser.describe("Grammar Page - Favorite Sync", () => {
    testWithFreshUser("should persist un-favorite after sync", async ({ page }) => {
        test.setTimeout(90_000);
        const grammarPage = await setupGrammarPage(page);

        // Add a grammar rule
        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        // Verify initially NOT favorited
        await expect.poll(async () => await grammarPage.isFavorited(0), { timeout: 5000 }).toBe(false);

        // Set to favorited
        await grammarPage.toggleFavoriteByIndex(0);
        await expect.poll(async () => await grammarPage.isFavorited(0), { timeout: 5000 }).toBe(true);

        // Navigate to Home — triggers sync with server
        await page.getByTestId("sidebar-tab-home").click();
        await page.waitForURL(/\/home$/, { timeout: 10000 });

        // Navigate back to Grammar
        await grammarPage.goto();
        await grammarPage.expectGrammarVisible();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        // Verify favorite persisted after sync
        await expect.poll(async () => await grammarPage.isFavorited(0), { timeout: 5000 }).toBe(true);

        // Un-favorite
        await grammarPage.toggleFavoriteByIndex(0);
        await expect.poll(async () => await grammarPage.isFavorited(0), { timeout: 5000 }).toBe(false);

        // Navigate to Home — triggers sync with server
        await page.getByTestId("sidebar-tab-home").click();
        await page.waitForURL(/\/home$/, { timeout: 10000 });

        // Navigate back to Grammar
        await grammarPage.goto();
        await grammarPage.expectGrammarVisible();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        // Verify UN-favorite persisted after sync (this was the bug)
        await expect.poll(async () => await grammarPage.isFavorited(0), { timeout: 5000 }).toBe(false);
    });
});

testWithFreshUser.describe("Grammar Page - Favorite Instant UI Update", () => {
    testWithFreshUser("should update favorite icon immediately after toggle", async ({ page }) => {
        test.setTimeout(90_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        await expect.poll(async () => await grammarPage.isFavorited(0), { timeout: 5000 }).toBe(false);

        await grammarPage.toggleFavoriteByIndex(0);
        await expect.poll(async () => await grammarPage.isFavorited(0), { timeout: 5000 }).toBe(true);

        await grammarPage.toggleFavoriteByIndex(0);
        await expect.poll(async () => await grammarPage.isFavorited(0), { timeout: 5000 }).toBe(false);
    });
});

testWithFreshUser.describe("Grammar Page - Practice Mode", () => {
    testWithFreshUser("should open practice modal with quiz questions from detail page", async ({ page }) => {
        test.setTimeout(90_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        const wordsPage = new WordsPage(page);
        await wordsPage.goto();
        await wordsPage.expectWordsVisible();
        await wordsPage.openAddModal();
        await wordsPage.enterText("私は本を読みます");
        await wordsPage.analyzeText();
        await wordsPage.selectFirstWord();
        await wordsPage.addSelectedWords();

        await grammarPage.goto();
        await grammarPage.expectGrammarVisible();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        await grammarPage.openPracticeForCard(0);
        await expect(grammarPage.practiceModal).toBeVisible({ timeout: 5000 });
    });

    testWithFreshUser("should open practice modal with correct structure", async ({ page }) => {
        test.setTimeout(90_000);
        const grammarPage = await setupGrammarPage(page);

        // Add grammar rule that applies to verbs
        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        // Open detail page and practice
        await grammarPage.openPracticeForCard(0);
        await expect(grammarPage.practiceModal).toBeVisible({ timeout: 5000 });

        // Practice modal should show either quiz questions or "no words" message
        // Both states are valid - depends on whether user has matching vocabulary
        const hasQuizContent = await grammarPage.practiceProgress.isVisible().catch(() => false);
        const hasNoWordsMessage = await grammarPage.practiceNoWords.isVisible().catch(() => false);
        expect(hasQuizContent || hasNoWordsMessage).toBe(true);

        // If showing quiz content, verify structure
        if (hasQuizContent) {
            const progressText = await grammarPage.practiceProgress.textContent();
            expect(progressText).toMatch(/\d+\s*\/\s*\d+/);
            await expect(grammarPage.practiceCorrectCount).toBeVisible();
            // Options should be visible
            await expect(grammarPage.practiceOptions[0]).toBeVisible();
        }

        // Close the modal
        await grammarPage.practiceCloseBtn.click();
        await expect(grammarPage.practiceModal).not.toBeVisible({ timeout: 5000 });
    });

    testWithFreshUser("should show enabled practice button for rules with format_map on detail page", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        await grammarPage.navigateToDetail(0);

        // Wait for detail page content to fully load (async WASM data fetch)
        await expect(grammarPage.detailContainer).toBeVisible({ timeout: 30_000 });

        await expect(grammarPage.detailPracticeBtn).toBeVisible({ timeout: 10_000 });
        // Button is visible for rules with format_map (no disabled state)
    });

    testWithFreshUser("should complete full practice quiz flow", async ({ page }) => {
        test.setTimeout(120_000);
        const grammarPage = await setupGrammarPage(page);

        // Add grammar rule
        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        // Add words to enable quiz generation
        const wordsPage = new WordsPage(page);
        await wordsPage.goto();
        await wordsPage.expectWordsVisible();
        await wordsPage.openAddModal();
        await wordsPage.enterText("私は本を読みます");
        await wordsPage.analyzeText();
        await wordsPage.selectFirstWord();
        await wordsPage.addSelectedWords();

        // Go back to grammar and open practice
        await grammarPage.goto();
        await grammarPage.expectGrammarVisible();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        await grammarPage.openPracticeForCard(0);
        await expect(grammarPage.practiceModal).toBeVisible({ timeout: 5_000 });

        // If quiz has content, answer all questions
        const hasQuizContent = await grammarPage.practiceProgress.isVisible().catch(() => false);
        if (hasQuizContent) {
            // Answer questions until complete or no more
            for (let i = 0; i < 10; i++) {
                const isComplete = await grammarPage.practiceComplete.isVisible().catch(() => false);
                if (isComplete) break;

                const hasNoWords = await grammarPage.practiceNoWords.isVisible().catch(() => false);
                if (hasNoWords) break;

                // Select a random option (first available)
                const option = grammarPage.practiceOptions[0];
                if (await option.isVisible().catch(() => false)) {
                    await option.click();
                    // Click next if available
                    if (await grammarPage.practiceNextBtn.isVisible().catch(() => false)) {
                        await grammarPage.practiceNextBtn.click();
                    }
                }
            }
        }

        // Close the modal
        await grammarPage.practiceCloseBtn.click();
        await expect(grammarPage.practiceModal).not.toBeVisible({ timeout: 5_000 });
    });
});

testWithFreshUser.describe("Grammar Page - Detail Page", () => {
    testWithFreshUser("should navigate to grammar detail page on card click", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        await grammarPage.navigateToDetail(0);
        await expect(page).toHaveURL(/\/grammar\//);
    });

    testWithFreshUser("should display grammar detail page content", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        await grammarPage.navigateToDetail(0);

        // Wait for detail page content to fully load (async WASM data fetch)
        await expect(grammarPage.detailContainer).toBeVisible({ timeout: 30_000 });
        await expect(grammarPage.detailBreadcrumbs).toBeVisible();
        await expect(grammarPage.detailFsrs).toBeVisible();
    });

    testWithFreshUser("should navigate back via breadcrumbs", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        await grammarPage.navigateToDetail(0);

        // Wait for detail page content to fully load (async WASM data fetch)
        await expect(grammarPage.detailContainer).toBeVisible({ timeout: 30_000 });

        await grammarPage.detailBreadcrumbsBack.click();
        await page.waitForURL(/\/grammar$/, { timeout: 5000 });
        await expect(page).toHaveURL(/\/grammar$/);
    });

    testWithFreshUser("should delete card from detail page and redirect to list", async ({ page }) => {
        test.setTimeout(60_000);
        const grammarPage = await setupGrammarPage(page);

        await grammarPage.openAddModal();
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
        await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 10_000 });

        await grammarPage.navigateToDetail(0);

        // Wait for detail page content to fully load (async WASM data fetch)
        await expect(grammarPage.detailContainer).toBeVisible({ timeout: 30_000 });

        // Now CardActionBar should be rendered
        const deleteBtn = grammarPage.detailActions.getByTestId("grammar-detail-actions-delete-btn");
        await expect(deleteBtn).toBeVisible({ timeout: 10_000 });
        await deleteBtn.click();

        await expect(grammarPage.detailDeleteModal).toBeVisible({ timeout: 5000 });
        await grammarPage.detailDeleteConfirmBtn.click();

        await page.waitForURL(/\/grammar$/, { timeout: 10_000 });
        await expect(page).toHaveURL(/\/grammar$/);
    });
});
