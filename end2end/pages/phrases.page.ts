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
    }

    async goto(): Promise<void> {
        await this.navigate("/phrases");
    }

    async expectPhrasesVisible(): Promise<void> {
        await expect(this.phrasesPage).toBeVisible();
        await expect(this.phrasesCard).toBeVisible();
        await expect(this.phrasesTitle).toBeVisible();
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
        return this.page.getByTestId("phrases-card-item").count();
    }

    async clickBack(): Promise<void> {
        await this.backButton.click();
    }
}
