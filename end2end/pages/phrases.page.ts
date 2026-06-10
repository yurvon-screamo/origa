import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

type PhrasesFilterType = "Все" | "Новые" | "Сложные" | "В процессе" | "Изученные";

export class PhrasesPage extends BasePage {
    readonly phrasesPage: Locator;
    readonly phrasesCard: Locator;
    readonly phrasesTitle: Locator;
    readonly backButton: Locator;
    readonly searchInput: Locator;
    readonly phrasesGrid: Locator;
    readonly emptyState: Locator;
    readonly loadMoreButton: Locator;
    readonly cardItem: Locator;

    readonly deleteModal: Locator;
    readonly deleteConfirmBtn: Locator;
    readonly deleteCancelBtn: Locator;

    constructor(page: Page) {
        super(page);

        this.phrasesPage = page.getByTestId("phrases-page");
        this.phrasesCard = page.getByTestId("phrases-card");
        this.phrasesTitle = page.getByTestId("phrases-title");
        this.backButton = page.getByTestId("phrases-back-btn");
        this.searchInput = page.getByTestId("phrases-search-input");
        this.phrasesGrid = page.getByTestId("phrases-grid");
        this.emptyState = page.getByTestId("phrases-empty-state");
        this.loadMoreButton = page.getByTestId("phrases-load-more-btn");
        this.cardItem = page.getByTestId("phrases-card-item");

        this.deleteModal = page.getByTestId("phrases-delete-modal");
        this.deleteConfirmBtn = page.getByTestId("phrases-delete-modal-confirm");
        this.deleteCancelBtn = page.getByTestId("phrases-delete-modal-cancel");
    }

    async goto(): Promise<void> {
        await this.navigate("/phrases");
    }

    async expectPhrasesVisible(timeout = 30_000): Promise<void> {
        // Leptos WASM hydration may take several seconds after navigation
        await expect(this.phrasesPage).toBeVisible({ timeout });
        await expect(this.phrasesCard).toBeVisible({ timeout });
        await expect(this.phrasesTitle).toBeVisible({ timeout });
    }

    async searchPhrases(query: string): Promise<void> {
        await this.searchInput.fill(query);
    }

    async selectFilter(name: PhrasesFilterType): Promise<void> {
        const filterMap: Record<PhrasesFilterType, string> = {
            "Все": "all",
            "Новые": "new",
            "Сложные": "hard",
            "В процессе": "in-progress",
            "Изученные": "learned",
        };
        await this.page.getByTestId("phrases-filter-" + filterMap[name]).click();
    }

    async getCardCount(): Promise<number> {
        return this.cardItem.count();
    }

    async clickBack(): Promise<void> {
        await this.backButton.click();
    }

    async markCardAsKnownByIndex(index: number): Promise<void> {
        await this.clickCardActionBtn(index, "phrases-card-item-mark-known-btn");
    }

    async getFavoriteButton(index: number): Promise<Locator> {
        const card = this.page.getByTestId("phrases-card-item").nth(index);
        return card.getByTestId("phrases-card-item-favorite-btn");
    }

    async toggleFavoriteByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("phrases-card-item").nth(index);
        const btn = card.getByTestId("phrases-card-item-favorite-btn");
        await btn.dispatchEvent("click");
        await this.page.waitForTimeout(1000);
    }

    async isFavorited(index: number): Promise<boolean> {
        const btn = await this.getFavoriteButton(index);
        const filledPath = btn.locator('svg path[fill="currentColor"]');
        return filledPath.isVisible().catch(() => false);
    }

    async deleteCardByIndex(index: number): Promise<void> {
        await this.clickCardActionBtn(index, "phrases-card-item-delete-btn");
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteConfirmBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 10_000 });
    }

    async cancelDeleteCardByIndex(index: number): Promise<void> {
        await this.clickCardActionBtn(index, "phrases-card-item-delete-btn");
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteCancelBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 5000 });
    }

    // Leptos event delegation requires bubbles: true for stop_propagation handlers
    private async clickCardActionBtn(index: number, btnTestId: string): Promise<void> {
        const selector = `[data-testid="phrases-card-item"]:nth-of-type(${index + 1}) [data-testid="${btnTestId}"]`;
        await this.page.evaluate((sel: string) => {
            const el = document.querySelector(sel) as HTMLElement;
            if (el) {
                el.dispatchEvent(new MouseEvent("click", { bubbles: true }));
            }
        }, selector);
    }
}
