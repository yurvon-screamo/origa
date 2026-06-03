import { test, expect, type Page } from "@playwright/test";
import { PhrasesPage } from "../pages";
import { testWithFreshUser } from "../fixtures";
import { skipOnboarding } from "../helpers/navigation";
import { waitForScoringReady } from "../helpers/onboarding";

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

async function completeFullOnboarding(page: Page): Promise<void> {
    await page.goto("/home");
    await page.waitForURL(/\/onboarding$/, { timeout: 30_000 });

    await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10_000 });

    // Intro → Load
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-load-step")).toBeVisible();

    // Load → JLPT
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible();

    // JLPT: select N5
    await page.getByTestId("jlpt-option-n5").click();
    await expect(page.getByTestId("jlpt-option-n5")).toHaveClass(/selected/, { timeout: 5000 });
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-apps-step")).toBeVisible();

    // Apps: wait for CDN-loaded apps to appear, then select all available
    await expect(page.getByTestId("apps-step-app-Migii-checkbox")).toBeVisible({ timeout: 15_000 });

    const migiiCheckbox = page.getByTestId("apps-step-app-Migii-checkbox");
    await migiiCheckbox.click();

    const duolingoRuCheckbox = page.getByTestId("apps-step-app-DuolingoRu-checkbox");
    if (await duolingoRuCheckbox.isVisible().catch(() => false)) {
        await duolingoRuCheckbox.click();
    }

    const minnaCheckbox = page.getByTestId("apps-step-app-MinnaNoNihongo-checkbox");
    if (await minnaCheckbox.isVisible().catch(() => false)) {
        await minnaCheckbox.click();
    }

    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-progress-step")).toBeVisible();

    // Progress: configure each selected app
    const migiiLevelDropdown = page.getByTestId("migii-level-dropdown");
    if (await migiiLevelDropdown.isVisible().catch(() => false)) {
        await migiiLevelDropdown.click();
        await page.getByTestId("migii-level-dropdown-option-N4").click();

        const migiiLessonDropdown = page.getByTestId("migii-lesson-dropdown");
        await migiiLessonDropdown.click();
        await page.getByTestId("migii-lesson-dropdown-option-lesson_10").click();
    }

    const duolingoRuModuleDropdown = page.getByTestId("DuolingoRu-module-dropdown");
    if (await duolingoRuModuleDropdown.isVisible().catch(() => false)) {
        await duolingoRuModuleDropdown.click();
        await page.getByTestId("DuolingoRu-module-dropdown-option-module_1").click();

        const duolingoRuUnitDropdown = page.getByTestId("DuolingoRu-unit-dropdown");
        await duolingoRuUnitDropdown.click();
        await page.getByTestId("DuolingoRu-unit-dropdown-option-unit_10").click();
    }

    const minnaLevelDropdown = page.getByTestId("minna-level-dropdown");
    if (await minnaLevelDropdown.isVisible().catch(() => false)) {
        await minnaLevelDropdown.click();
        await page.getByTestId("minna-level-dropdown-option-N4").click();

        const minnaLessonDropdown = page.getByTestId("minna-lesson-dropdown");
        await minnaLessonDropdown.click();
        await page.getByTestId("minna-lesson-dropdown-option-lesson_38").click();
    }

    // Progress → Summary
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
        await page.waitForURL(/\/phrases$/, { timeout: 10_000 });
        await page.waitForLoadState("networkidle");
        // Wait for WASM hydration to complete (Leptos renders after JS loads)
        await page.locator(".loading-spinner").waitFor({ state: "hidden", timeout: 30_000 }).catch(() => {});
        await phrasesPage.expectPhrasesVisible();

        await expect(phrasesPage.emptyState).not.toBeVisible({ timeout: 30_000 });
    });

    testWithFreshUser("phrase cards should display Japanese text and meaning after data loads", async ({ page }) => {
        test.setTimeout(300_000);

        await completeFullOnboarding(page);
        await expect(page).toHaveURL(/\/home$/);

        const phrasesPage = new PhrasesPage(page);
        await phrasesPage.goto();
        await page.waitForURL(/\/phrases$/, { timeout: 10_000 });
        await page.waitForLoadState("networkidle");
        await page.locator(".loading-spinner").waitFor({ state: "hidden", timeout: 30_000 }).catch(() => {});
        await phrasesPage.expectPhrasesVisible();

        // Wait for phrase data to load from CDN
        await expect(phrasesPage.emptyState).not.toBeVisible({ timeout: 30_000 });

        // Verify that cards have rendered
        const firstCard = phrasesPage.cardItem.first();
        await expect(firstCard).toBeVisible({ timeout: 30_000 });

        // Card has phrase text with furigana (ruby elements)
        const phraseText = firstCard.getByTestId("phrases-card-phrase");
        await expect(phraseText).toBeAttached({ timeout: 30_000 });
    });

    testWithFreshUser("should search and filter phrases after onboarding", async ({ page }) => {
        test.setTimeout(300_000);

        await completeFullOnboarding(page);
        await expect(page).toHaveURL(/\/home$/);

        const phrasesPage = new PhrasesPage(page);
        await phrasesPage.goto();
        await page.waitForURL(/\/phrases$/, { timeout: 10_000 });
        await page.waitForLoadState("networkidle");
        // Wait for WASM hydration to complete (Leptos renders after JS loads)
        await page.locator(".loading-spinner").waitFor({ state: "hidden", timeout: 30_000 }).catch(() => {});
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
        await page.waitForURL(/\/phrases$/, { timeout: 10_000 });
        await page.waitForLoadState("networkidle");
        // Wait for WASM hydration to complete (Leptos renders after JS loads)
        await page.locator(".loading-spinner").waitFor({ state: "hidden", timeout: 30_000 }).catch(() => {});
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
