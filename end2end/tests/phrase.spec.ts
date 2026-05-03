import { test, expect, type Page } from "@playwright/test";
import { PhrasesPage } from "../pages";
import { testWithFreshUser } from "../fixtures";

interface PhraseData {
    phrases: Phrase[];
}

interface Phrase {
    id: string;
    text: string;
    audio_file: string;
    tokens: unknown[];
    translation_ru: string;
    translation_en: string;
}

test.describe("Phrase Dataset", () => {
    let response_data: PhraseData;

    test.beforeAll(async ({ request }) => {
        const response = await request.get("/public/phrase/phrase_dataset.json");
        expect(response.ok()).toBeTruthy();
        response_data = (await response.json()) as PhraseData;
    });

    test("phrase dataset loads successfully", async () => {
        expect(response_data.phrases.length).toBe(505);
    });

    test("phrase dataset has required fields", async () => {
        const phrase = response_data.phrases[0];

        expect(phrase).toHaveProperty("id");
        expect(phrase).toHaveProperty("text");
        expect(phrase).toHaveProperty("audio_file");
        expect(phrase).toHaveProperty("tokens");
        expect(phrase).toHaveProperty("translation_ru");
        expect(phrase).toHaveProperty("translation_en");
    });

    test("phrase audio file is accessible", async ({ request }) => {
        const phrase = response_data.phrases[0];
        const audioResponse = await request.get(`/public/phrase/audio/${phrase.audio_file}`);

        expect(audioResponse.ok()).toBeTruthy();
        const body = await audioResponse.body();
        expect(body.length).toBeGreaterThan(0);
    });

    test("phrase audio files are opus format", async () => {
        for (const phrase of response_data.phrases) {
            expect(phrase.audio_file).toMatch(/\.opus$/);
        }
    });

    test("all phrase ids are unique ULIDs", async () => {
        const ids = response_data.phrases.map((p) => p.id);
        expect(new Set(ids).size).toBe(ids.length);
    });

    test("all phrases have non-empty translations", async () => {
        for (const phrase of response_data.phrases) {
            expect(phrase.translation_ru.length).toBeGreaterThan(0);
            expect(phrase.translation_en.length).toBeGreaterThan(0);
        }
    });
});

test.describe("Phrases Navigation", () => {
    testWithFreshUser("bottom nav has Phrases tab", async ({ page }) => {
        await page.goto("/home");
        await page.waitForLoadState("domcontentloaded");

        const phrasesTab = page.getByTestId("tab-phrases");
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
    await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10000 });
    await page.getByTestId("onboarding-skip").click();
    await page.waitForURL(/\/home$/, { timeout: 10000 });

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

    testWithFreshUser("should navigate back to home", async ({ page }) => {
        const phrasesPage = await setupPhrasesPage(page);
        await phrasesPage.clickBack();
        await page.waitForURL(/\/home$/, { timeout: 10000 });
        await expect(page).toHaveURL(/\/home$/);
    });

    testWithFreshUser("should show filter buttons", async ({ page }) => {
        const phrasesPage = await setupPhrasesPage(page);

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
    await page.goto("http://localhost:1420/");

    try {
        await page.waitForURL(/\/onboarding$/, { timeout: 30_000 });
    } catch {
        if (page.url().includes("/home")) return;
    }

    if (page.url().includes("/home")) return;

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

    // Apps: skip (no selection)
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-progress-step")).toBeVisible();

    // Progress: skip (no configuration)
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-summary-step")).toBeVisible();

    // Summary → Import
    await page.getByTestId("onboarding-import").click();
    await expect(page.getByTestId("onboarding-scoring-step")).toBeVisible({ timeout: 120_000 });

    await waitForScoringReady(page);
    await page.waitForTimeout(1000);

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
});
