import { test, expect, type Page } from "@playwright/test";
import { PhrasesPage } from "../pages";
import { testWithFreshUser } from "../fixtures";
import { skipOnboarding } from "../helpers/navigation";

test.describe("Phrases Navigation", () => {
    testWithFreshUser("bottom nav has Phrases tab", async ({ page }) => {
        await page.setViewportSize({ width: 375, height: 667 });
        await page.goto("/home");
        await page.waitForLoadState("domcontentloaded");

        const phrasesTab = page.getByTestId("bottom-tab-tab-phrases");
        await expect(phrasesTab).toBeVisible({ timeout: 10000 });
    });

    testWithFreshUser("navigating to /phrases shows phrases page", async ({ page }) => {
        await page.goto("/phrases");
        await page.waitForLoadState("domcontentloaded");

        const url = page.url();
        expect(url).toContain("phrases");
    });
});

async function setupPhrasesPage(page: Page): Promise<PhrasesPage> {
    await skipOnboarding(page);

    const phrasesPage = new PhrasesPage(page);
    await phrasesPage.goto();
    await phrasesPage.expectPhrasesVisible();
    return phrasesPage;
}

testWithFreshUser.describe("Phrases Page", () => {
    testWithFreshUser("should display empty state for new user", async ({ page }) => {
        const phrasesPage = await setupPhrasesPage(page);
        await expect(phrasesPage.emptyState).toBeVisible();
    });

    testWithFreshUser("should navigate to home via sidebar", async ({ page }) => {
        await setupPhrasesPage(page);
        await page.getByTestId("sidebar-tab-home").click();
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
        await expect(page).toHaveURL(/\/home$/);
    });

    testWithFreshUser("should show filter buttons", async ({ page }) => {
        await setupPhrasesPage(page);

        await expect(page.getByTestId("phrases-filter-all")).toBeVisible();
        await expect(page.getByTestId("phrases-filter-new")).toBeVisible();
        await expect(page.getByTestId("phrases-filter-learned")).toBeVisible();
    });

    testWithFreshUser("should show search input", async ({ page }) => {
        const phrasesPage = await setupPhrasesPage(page);
        await expect(phrasesPage.searchInput).toBeVisible();
    });
});

async function waitForScoringReady(page: Page, timeout = 30_000): Promise<void> {
    await Promise.race([
        page.getByTestId("scoring-step-hint").waitFor({ state: "visible", timeout }),
        page.getByTestId("scoring-step-complete").waitFor({ state: "visible", timeout }),
    ]).catch(() => {});
}

async function completeFullOnboarding(page: Page): Promise<void> {
    await page.goto("/home");
    await page.waitForURL(/\/onboarding$/, { timeout: 30_000 });

    await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10_000 });

    // Intro → Load
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-load-step")).toBeVisible();

    // Load → JLPT (default medium load)
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible();

    // JLPT: select N5
    await page.getByTestId("jlpt-option-n5").click();
    await expect(page.getByTestId("jlpt-option-n5")).toHaveClass(/selected/, { timeout: 5000 });
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-apps-step")).toBeVisible();

    // Apps: try selecting Migii if available
    const migiiCheckbox = page.getByTestId("apps-step-app-Migii-checkbox");
    if (await migiiCheckbox.isVisible().catch(() => false)) {
        await migiiCheckbox.click();
    }

    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-progress-step")).toBeVisible();

    // Progress: skip
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-summary-step")).toBeVisible();

    // Summary → Import
    const importBtn = page.getByTestId("onboarding-import");
    await importBtn.click();
    await expect(page.getByTestId("onboarding-scoring-step")).toBeVisible({ timeout: 120_000 });

    await waitForScoringReady(page);

    // Mark all known
    await page.getByTestId("onboarding-mark-all-known").click();
    await expect(page.getByTestId("scoring-step-complete")).toBeVisible({ timeout: 60_000 });

    // Finish
    await page.getByTestId("onboarding-finish").click();
    await page.waitForURL(/\/home$/, { timeout: 30_000 });
}

testWithFreshUser.describe("Phrases after full onboarding", () => {
    testWithFreshUser("should show phrase cards after completing onboarding with N5", async ({ page }) => {
        test.setTimeout(300_000);

        await completeFullOnboarding(page);
        await expect(page).toHaveURL(/\/home$/);

        const phrasesPage = new PhrasesPage(page);
        await phrasesPage.goto();
        await phrasesPage.expectPhrasesVisible();

        // CDN loading may take time for phrase data
        await expect(phrasesPage.emptyState).not.toBeVisible({ timeout: 30_000 });
    });

    testWithFreshUser("phrase cards should display Japanese text and meaning after data loads", async ({ page }) => {
        test.setTimeout(300_000);

        await completeFullOnboarding(page);
        await expect(page).toHaveURL(/\/home$/);

        const phrasesPage = new PhrasesPage(page);
        await phrasesPage.goto();
        await phrasesPage.expectPhrasesVisible();

        // Wait for phrase data to load from CDN
        await expect(phrasesPage.emptyState).not.toBeVisible({ timeout: 30_000 });

        // Verify that at least one card has rendered with content
        const firstCard = phrasesPage.cardItem.first();
        await expect(firstCard).toBeVisible({ timeout: 30_000 });

        // CRITICAL: Verify phrase text is NOT empty (reproduces the bug)
        const phraseText = firstCard.getByTestId("phrases-card-text");
        await expect(phraseText).toContainText(/\S/, { timeout: 30_000 });

        // Verify meaning/translation is present
        const meaning = firstCard.getByTestId("phrases-card-meaning");
        await expect(meaning).toContainText(/\S/, { timeout: 30_000 });
    });

    testWithFreshUser("should search and filter phrases after onboarding", async ({ page }) => {
        test.setTimeout(300_000);

        await completeFullOnboarding(page);
        await expect(page).toHaveURL(/\/home$/);

        const phrasesPage = new PhrasesPage(page);
        await phrasesPage.goto();
        await phrasesPage.expectPhrasesVisible();

        await expect(phrasesPage.emptyState).not.toBeVisible({ timeout: 30_000 });
        const countAll = await phrasesPage.getCardCount();
        expect(countAll).toBeGreaterThan(0);

        // Search with non-matching query
        await phrasesPage.searchPhrases("xyznonexistent");
        await expect(phrasesPage.emptyState).toBeVisible({ timeout: 5_000 });

        // Clear search — cards should reappear
        await phrasesPage.searchPhrases("");
        await expect(phrasesPage.emptyState).not.toBeVisible({ timeout: 5_000 });
        const countAfterClear = await phrasesPage.getCardCount();
        expect(countAfterClear).toBe(countAll);
    });

    testWithFreshUser("should filter phrases by status after onboarding", async ({ page }) => {
        test.setTimeout(300_000);

        await completeFullOnboarding(page);
        await expect(page).toHaveURL(/\/home$/);

        const phrasesPage = new PhrasesPage(page);
        await phrasesPage.goto();
        await phrasesPage.expectPhrasesVisible();

        await expect(phrasesPage.emptyState).not.toBeVisible({ timeout: 30_000 });

        // "New" filter should show phrases (all are new after onboarding)
        await phrasesPage.selectFilter("Новые");
        const newCount = await phrasesPage.getCardCount();
        expect(newCount).toBeGreaterThan(0);

        // "Learned" filter should show empty (nothing learned yet)
        await phrasesPage.selectFilter("Изученные");
        await expect(phrasesPage.emptyState).toBeVisible({ timeout: 5_000 });
    });
});
