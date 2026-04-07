import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

type WordsFilterType = "Все" | "Новые" | "Сложные" | "В процессе" | "Изученные";

export class WordsPage extends BasePage {
    // Page structure
    readonly wordsPage: Locator;
    readonly wordsCard: Locator;
    readonly wordsTitle: Locator;

    // Navigation buttons
    readonly backButton: Locator;
    readonly setsButton: Locator;
    readonly addButton: Locator;

    // Search
    readonly searchInput: Locator;

    // Content
    readonly wordsGrid: Locator;
    readonly emptyState: Locator;

    // Add-words drawer
    readonly drawer: Locator;
    readonly drawerTextarea: Locator;
    readonly drawerAnalyzeBtn: Locator;
    readonly drawerAddBtn: Locator;
    readonly drawerCancelBtn: Locator;
    readonly analyzedWordItems: Locator;

    // Anki import
    readonly ankiTab: Locator;
    readonly ankiDropZone: Locator;
    readonly ankiFileInput: Locator;
    readonly ankiFieldWord: Locator;
    readonly ankiFieldTranslation: Locator;
    readonly ankiNextBtn: Locator;
    readonly ankiBackBtn: Locator;
    readonly ankiImportBtn: Locator;
    readonly ankiCardCount: Locator;
    readonly ankiCardList: Locator;
    readonly ankiDone: Locator;
    readonly ankiError: Locator;
    readonly ankiRetryBtn: Locator;

    // Delete modal
    readonly deleteModal: Locator;
    readonly deleteConfirmBtn: Locator;
    readonly deleteCancelBtn: Locator;

    constructor(page: Page) {
        super(page);

        // Page structure
        this.wordsPage = page.getByTestId("words-page");
        this.wordsCard = page.getByTestId("words-card");
        this.wordsTitle = page.getByTestId("words-title");

        // Navigation buttons
        this.backButton = page.getByTestId("words-back-btn");
        this.setsButton = page.getByTestId("words-sets-btn");
        this.addButton = page.getByTestId("words-add-btn");

        // Search
        this.searchInput = page.getByTestId("words-search-input");

        // Content
        this.wordsGrid = page.getByTestId("words-grid");
        this.emptyState = page.getByTestId("words-empty-state");

        // Add-words drawer
        this.drawer = page.getByTestId("words-add-drawer");
        this.drawerTextarea = page.getByTestId("words-drawer-textarea");
        this.drawerAnalyzeBtn = page.getByTestId("words-drawer-analyze-btn");
        this.drawerAddBtn = page.getByTestId("words-drawer-add-btn");
        this.drawerCancelBtn = page.getByTestId("words-drawer-cancel-btn");
        this.analyzedWordItems = this.drawer.getByTestId("words-drawer-item");

        // Anki import
        this.ankiTab = this.drawer.getByText("Anki");
        this.ankiDropZone = page.getByTestId("anki-import-drop-zone");
        this.ankiFileInput = page.getByTestId("anki-import-file-input");
        this.ankiFieldWord = page.getByTestId("anki-import-field-word");
        this.ankiFieldTranslation = page.getByTestId("anki-import-field-translation");
        this.ankiNextBtn = page.getByTestId("anki-import-next-btn");
        this.ankiBackBtn = page.getByTestId("anki-import-back-btn");
        this.ankiImportBtn = page.getByTestId("anki-import-import-btn");
        this.ankiCardCount = page.getByTestId("anki-import-card-count");
        this.ankiCardList = page.getByTestId("anki-import-card-list");
        this.ankiDone = page.getByTestId("anki-import-done");
        this.ankiError = page.getByTestId("anki-import-error");
        this.ankiRetryBtn = page.getByTestId("anki-import-retry-btn");

        // Delete modal
        this.deleteModal = page.getByTestId("words-delete-modal");
        this.deleteConfirmBtn = page.getByTestId("words-delete-modal-confirm");
        this.deleteCancelBtn = page.getByTestId("words-delete-modal-cancel");
    }

    async goto(): Promise<void> {
        await this.navigate("/words");
    }

    async expectWordsVisible(): Promise<void> {
        await expect(this.wordsPage).toBeVisible();
        await expect(this.wordsCard).toBeVisible();
        await expect(this.wordsTitle).toBeVisible();
    }

    async searchWords(query: string): Promise<void> {
        await this.searchInput.fill(query);
    }

    async clickBack(): Promise<void> {
        await this.backButton.click();
    }

    async clickSets(): Promise<void> {
        await this.setsButton.click();
    }

    async openAddModal(): Promise<void> {
        await this.addButton.click();
        await expect(this.drawer).toBeVisible({ timeout: 5000 });
    }

    async enterText(text: string): Promise<void> {
        await this.drawerTextarea.waitFor({ state: "visible", timeout: 5000 });
        await this.drawerTextarea.fill(text);
    }

    async analyzeText(): Promise<void> {
        await this.drawerAnalyzeBtn.click();
        // Wait for analysis results - the "Найдено" text indicates completion
        await this.drawer.getByText(/Найдено/).waitFor({ state: "visible", timeout: 10_000 });
    }

    async selectFirstWord(): Promise<void> {
        const firstItem = this.analyzedWordItems.first();
        await firstItem.waitFor({ state: "visible", timeout: 5000 });
        await firstItem.click();
    }

    async addSelectedWords(): Promise<void> {
        await this.drawerAddBtn.click({ timeout: 5000 });
        await expect(this.drawer).not.toBeVisible({ timeout: 15_000 });
    }

    async cancelAddModal(): Promise<void> {
        await this.drawerCancelBtn.click();
    }

    async switchToAnkiTab(): Promise<void> {
        await this.ankiTab.click();
    }

    async uploadAnkiFile(filePath: string): Promise<void> {
        await this.ankiFileInput.setInputFiles(filePath);
    }

    async selectFilter(name: WordsFilterType): Promise<void> {
        const filterMap: Record<WordsFilterType, string> = {
            "Все": "all",
            "Новые": "new",
            "Сложные": "hard",
            "В процессе": "in-progress",
            "Изученные": "learned",
        };
        await this.page.getByTestId("words-filter-" + filterMap[name]).click();
    }

    async getCardCount(): Promise<number> {
        return this.page.getByTestId("words-card-item").count();
    }

    async deleteCardByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("words-card-item").nth(index);
        await card.getByTestId("words-card-item-delete-btn").click();
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteConfirmBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 10_000 });
    }

    async cancelDeleteCardByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("words-card-item").nth(index);
        await card.getByTestId("words-card-item-delete-btn").click();
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteCancelBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 5000 });
    }

    async markCardAsKnownByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("words-card-item").nth(index);
        await card.getByTestId("words-card-item-mark-known-btn").click();
        await this.page.waitForTimeout(500);
    }
}
