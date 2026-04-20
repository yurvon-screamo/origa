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
    test("bottom nav has Phrases tab", async ({ page }) => {
        await page.goto("/home");
        await page.waitForLoadState("domcontentloaded");

        const phrasesTab = page.getByTestId("tab-phrases");
        await expect(phrasesTab).toBeVisible({ timeout: 10000 });
    });

    test("navigating to /phrases shows phrases page", async ({ page }) => {
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
