import { Page, Locator, expect } from "@playwright/test";
import { CardListPage } from "./card_list.page";

type PhrasesFilterType = "Все" | "Новые" | "Сложные" | "В процессе" | "Изученные";

export class PhrasesPage extends CardListPage {
    // Legacy-prefixed aliases — the phrases steps reference these names.
    readonly phrasesPage: Locator;
    readonly phrasesCard: Locator;
    readonly phrasesTitle: Locator;
    readonly backButton: Locator;
    readonly phrasesGrid: Locator;

    constructor(page: Page) {
        super(page, "phrases");

        this.phrasesPage = this.pageRoot;
        this.phrasesCard = this.card;
        this.phrasesTitle = this.title;
        this.backButton = page.getByTestId("phrases-back-btn");
        this.phrasesGrid = this.grid;
    }

    async goto(): Promise<void> {
        await this.navigate("/phrases");
    }

    async expectPhrasesVisible(timeout = 30_000): Promise<void> {
        await this.expectVisible(timeout);
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
        const btn = this.cardItem.nth(index).getByTestId("phrases-card-item-mark-known-btn");
        await btn.dispatchEvent("click");
    }

    async deleteCardByIndex(index: number): Promise<void> {
        await this.cardItem.nth(index).getByTestId("phrases-card-item-delete-btn").dispatchEvent("click");
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteConfirmBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 10_000 });
    }

    async cancelDeleteCardByIndex(index: number): Promise<void> {
        await this.cardItem.nth(index).getByTestId("phrases-card-item-delete-btn").dispatchEvent("click");
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteCancelBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 5000 });
    }
}
