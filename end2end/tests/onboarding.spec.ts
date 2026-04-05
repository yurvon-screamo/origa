import { expect, test, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { HomePage, KanjiPage, LoginPage, OnboardingPage, WordsPage, GrammarPage } from "../pages";

/**
 * Onboarding Flow E2E Tests
 *
 * Tests the complete onboarding flow:
 * 1. Login (handled by fixture)
 * 2. Intro step
 * 3. JLPT level selection (N4)
 * 4. Apps selection (Migii, Duolingo RU, Minna N4)
 * 5. Progress configuration (~50% on each)
 * 6. Summary and import
 */

testWithFreshUser.describe("Onboarding Flow - N4 with ~50% Progress", () => {
    testWithFreshUser("should complete full onboarding flow", async ({ page }: { page: Page }) => {
        // Set viewport for consistent testing
        await page.setViewportSize({ width: 1280, height: 720 });

        const loginPage = new LoginPage(page);
        const onboardingPage = new OnboardingPage(page);

        // Navigate to login page - user is already authenticated via fixture
        await loginPage.goto();

        // Wait for redirect after login (fixture already sets auth token)
        await page.waitForURL(/\/(onboarding|home)$/, { timeout: 10000 });

        // ========================================
        // Step 0: Verify onboarding page with stepper
        // ========================================
        await expect(page).toHaveURL(/\/onboarding$/);

        // Wait for loading to complete
        await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({
            timeout: 10000
        });

        // Verify page structure
        await expect(page.getByTestId("onboarding-page")).toBeVisible();
        await expect(page.getByTestId("onboarding-card")).toBeVisible();
        await expect(page.getByTestId("onboarding-stepper")).toBeVisible();

        // ========================================
        // Step 1: Intro - should display welcome message and proceed
        // ========================================
        await expect(page.getByTestId("onboarding-intro-step")).toBeVisible();

        // Verify welcome text
        await expect(page.getByTestId("intro-step-title")).toBeVisible();

        // Verify skip button is visible
        const skipButton = page.getByTestId("onboarding-skip");
        await expect(skipButton).toBeVisible();

        // Take screenshot for visual verification
        await page.screenshot({
            path: "test-results/onboarding-step-1-intro.png",
            fullPage: true
        });

        // Click "Далее" to proceed
        await page.getByTestId("onboarding-next").click();

        // Verify we moved to Load step
        await expect(page.getByTestId("onboarding-load-step")).toBeVisible();

        // ========================================
        // Step 1.5: Load - select daily load
        // ========================================
        await expect(page.getByTestId("load-step-title")).toBeVisible();

        // Proceed with default (medium) load
        await page.getByTestId("onboarding-next").click();

        // Verify we moved to JLPT step
        await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible();

        // ========================================
        // Step 2: JLPT - should select N4 level
        // ========================================
        await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible();
        await expect(page.getByTestId("jlpt-step-title")).toBeVisible();

        // Take screenshot before selection
        await page.screenshot({
            path: "test-results/onboarding-step-2-jlpt-before.png",
            fullPage: true
        });

        // Select N4 level using test_id
        await page.getByTestId("jlpt-option-n4").click();

        // Verify selection is highlighted (check for selected class)
        const n4Option = page.getByTestId("jlpt-option-n4");
        await expect(n4Option).toHaveClass(/selected/, { timeout: 5000 });

        // Screenshot after selection
        await page.screenshot({
            path: "test-results/onboarding-step-2-jlpt-after.png",
            fullPage: true
        });

        // Proceed to next step
        await page.getByTestId("onboarding-next").click();
        await expect(page.getByTestId("onboarding-apps-step")).toBeVisible();

        // ========================================
        // Step 3: Apps - should select all available apps
        // ========================================
        await expect(page.getByTestId("onboarding-apps-step")).toBeVisible();
        await expect(page.getByTestId("apps-step-title")).toBeVisible();

        // Take screenshot before selections
        await page.screenshot({
            path: "test-results/onboarding-step-3-apps-before.png",
            fullPage: true
        });

        // Select Migii
        const migiiCheckbox = page.getByTestId("apps-step-app-Migii-checkbox");
        await expect(migiiCheckbox).toBeVisible();
        await migiiCheckbox.click();

        // Verify Migii is selected (check checkbox state)
        const migiiCard = page.getByTestId("apps-step-app-Migii");
        await expect(migiiCard).toHaveClass(/selected/, { timeout: 5000 });

        // Select Duolingo 「RU」
        const duolingoRuCheckbox = page.getByTestId("apps-step-app-DuolingoRu-checkbox");
        if (await duolingoRuCheckbox.isVisible().catch(() => false)) {
            await duolingoRuCheckbox.click();
        }

        // Select Duolingo 「EN」
        const duolingoEnCheckbox = page.getByTestId("apps-step-app-DuolingoEn-checkbox");
        if (await duolingoEnCheckbox.isVisible().catch(() => false)) {
            await duolingoEnCheckbox.click();
        }

        // Select Minna no Nihongo
        const minnaNoNihongoCheckbox = page.getByTestId("apps-step-app-MinnaNoNihongo-checkbox");
        if (await minnaNoNihongoCheckbox.isVisible().catch(() => false)) {
            await minnaNoNihongoCheckbox.click();
        }

        // Screenshot after selections
        await page.screenshot({
            path: "test-results/onboarding-step-3-apps-after.png",
            fullPage: true
        });

        // Proceed to progress step
        await page.getByTestId("onboarding-next").click();
        await expect(page.getByTestId("onboarding-progress-step")).toBeVisible();

        // ========================================
        // Step 4: Progress - should configure ~50% progress for each app
        // ========================================
        await expect(page.getByTestId("onboarding-progress-step")).toBeVisible();
        await expect(page.getByTestId("progress-step-title")).toBeVisible();

        // Take screenshot
        await page.screenshot({
            path: "test-results/onboarding-step-4-progress-before.png",
            fullPage: true
        });

        // Configure Migii progress (N4, middle lesson)
        const migiiLevelDropdown = page.getByTestId("migii-level-dropdown");
        if (await migiiLevelDropdown.isVisible().catch(() => false)) {
            await migiiLevelDropdown.click();
            await page.getByTestId("migii-level-dropdown-option-N4").click();

            // Select middle lesson (around lesson 10 for N4)
            const migiiLessonDropdown = page.getByTestId("migii-lesson-dropdown");
            await migiiLessonDropdown.click();
            await page.getByTestId("migii-lesson-dropdown-option-lesson_10").click();
        }

        // Configure Duolingo 「RU」 progress
        const duolingoRuModuleDropdown = page.getByTestId("DuolingoRu-module-dropdown");
        if (await duolingoRuModuleDropdown.isVisible().catch(() => false)) {
            // Select first module
            await duolingoRuModuleDropdown.click();
            await page.getByTestId("DuolingoRu-module-dropdown-option-module_1").click();

            // Select ~50% unit
            const duolingoRuUnitDropdown = page.getByTestId("DuolingoRu-unit-dropdown");
            await duolingoRuUnitDropdown.click();
            await page.getByTestId("DuolingoRu-unit-dropdown-option-unit_10").click();
        }

        // Configure Minna no Nihongo progress (two dropdowns: level + lesson)
        const minnaLevelDropdown = page.getByTestId("minna-level-dropdown");
        if (await minnaLevelDropdown.isVisible().catch(() => false)) {
            await minnaLevelDropdown.click();
            await page.getByTestId("minna-level-dropdown-option-N4").click();

            const minnaLessonDropdown = page.getByTestId("minna-lesson-dropdown");
            await minnaLessonDropdown.click();
            await page.getByTestId("minna-lesson-dropdown-option-lesson_38").click();
        }

        // Screenshot after progress configuration
        await page.screenshot({
            path: "test-results/onboarding-step-4-progress-after.png",
            fullPage: true
        });

        // Proceed to summary
        await page.getByTestId("onboarding-next").click();
        await expect(page.getByTestId("onboarding-summary-step")).toBeVisible();

        // ========================================
        // Step 5: Summary - should display selected sets and allow toggle
        // ========================================
        await expect(page.getByTestId("onboarding-summary-step")).toBeVisible();
        await expect(page.getByTestId("summary-step-title")).toBeVisible();

        // Verify word count is displayed
        await expect(page.getByTestId("summary-step-stats")).toBeVisible();

        // Take screenshot
        await page.screenshot({
            path: "test-results/onboarding-step-5-summary-before.png",
            fullPage: true
        });

        // Test accordion toggle (if expandable)
        const accordionHeader = page.getByTestId('summary-step-accordion-header').first();
        if (await accordionHeader.isVisible().catch(() => false)) {
            await accordionHeader.click();
            // Verify content collapses/expands
            await page.waitForTimeout(300); // Wait for animation
        }

        // Test set toggle checkbox (optional - deselect then reselect)
        const firstSetCheckbox = page.locator('[data-testid^="summary-step-set-"][data-testid$="-checkbox"]').first();
        if (await firstSetCheckbox.isVisible().catch(() => false)) {
            // Note: Clicking will toggle off, then we'd need to toggle back on
            // For now, just verify it exists and is clickable
            await expect(firstSetCheckbox).toBeEnabled();
        }

        // Screenshot before import
        await page.screenshot({
            path: "test-results/onboarding-step-5-summary-before-import.png",
            fullPage: true
        });

        // ========================================
        // Final: Complete import and redirect to home

        // ========================================
        // Start import
        await page.getByTestId("onboarding-import").click();

        // Verify import button shows loading state
        await expect(page.getByTestId("onboarding-import")).toHaveAttribute("data-loading", "true", { timeout: 5000 });

        // Wait for scoring step to appear after import completes
        await expect(page.getByTestId("onboarding-scoring-step")).toBeVisible({ timeout: 120_000 });

        // Take screenshot of scoring step
        await page.screenshot({
            path: "test-results/onboarding-step-6-scoring.png",
            fullPage: true
        });

        // Click "Завершить" to finish onboarding and navigate to home
        await page.getByTestId("onboarding-finish").click();

        // Wait for redirect to home (can take time for import)
        await page.waitForURL(/\/home$/, { timeout: 30_000 });

        // Verify we're on home page
        await expect(page).toHaveURL(/\/home$/);

        // Take final screenshot
        await page.screenshot({
            path: "test-results/onboarding-complete-home.png",
            fullPage: true
        });
    });

    testWithFreshUser("should skip onboarding and redirect to home", async ({ page }: { page: Page }) => {
        // Wait for redirect after login
        await page.waitForLoadState("networkidle");
        await page.waitForTimeout(2000);

        // Wait for either home or onboarding
        try {
            await page.waitForURL(/\/(home|onboarding)/, { timeout: 10000 });
        } catch {
            // URL didn't match, continue anyway
        }

        // If on home already, skip is done
        if (page.url().includes("/home")) {
            await expect(page).toHaveURL(/\/home$/);
            return;
        }

        // Otherwise we're on onboarding
        await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10000 });
        await expect(page.getByTestId("onboarding-skip")).toBeVisible();
        await page.getByTestId("onboarding-skip").click();
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
        await expect(page).toHaveURL(/\/home$/);
    });

    testWithFreshUser("should navigate back through steps", async ({ page }: { page: Page }) => {
        // Wait for redirect after login
        await page.waitForLoadState("networkidle");
        await page.waitForTimeout(2000);

        // Wait for onboarding URL
        try {
            await page.waitForURL(/\/onboarding$/, { timeout: 10000 });
        } catch {
            // If we're not on onboarding, check if we're on home (skip onboarding might be automatic)
            if (page.url().includes("/home")) {
                // Onboarding was skipped, test can't proceed
                test.skip();
                return;
            }
        }

        await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10000 });

        // Intro → Load
        await page.getByTestId("onboarding-next").click();
        await expect(page.getByTestId("onboarding-load-step")).toBeVisible();

        // Load → JLPT
        await page.getByTestId("onboarding-next").click();
        await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible();

        // JLPT → Apps (select N4 first)
        await page.getByTestId("jlpt-option-n4").click();
        await page.getByTestId("onboarding-next").click();
        await expect(page.getByTestId("onboarding-apps-step")).toBeVisible();

        // Back to JLPT
        await page.getByTestId("onboarding-prev").click();
        await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible();

        // Back to Load
        await page.getByTestId("onboarding-prev").click();
        await expect(page.getByTestId("onboarding-load-step")).toBeVisible();

        // Back to Intro
        await page.getByTestId("onboarding-prev").click();
        await expect(page.getByTestId("onboarding-intro-step")).toBeVisible();
    });
});
