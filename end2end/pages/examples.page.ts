import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class ExamplesPage extends BasePage {
    readonly examplesPage: Locator;
    readonly examplesCard: Locator;
    readonly examplesTitle: Locator;
    readonly backButton: Locator;
    readonly searchInput: Locator;
    readonly examplesGrid: Locator;
    readonly emptyState: Locator;
    readonly loadMoreButton: Locator;
    readonly cardItem: Locator;

    readonly deleteModal: Locator;
    readonly deleteConfirmBtn: Locator;
    readonly deleteCancelBtn: Locator;

    constructor(page: Page) {
        super(page);

        this.examplesPage = page.getByTestId("examples-page");
        this.examplesCard = page.getByTestId("examples-card");
        this.examplesTitle = page.getByTestId("examples-title");
        this.backButton = page.getByTestId("examples-back-btn");
        this.searchInput = page.getByTestId("examples-search-input");
        this.examplesGrid = page.getByTestId("examples-grid");
        this.emptyState = page.getByTestId("examples-empty-state");
        this.loadMoreButton = page.getByTestId("examples-load-more-btn");
        this.cardItem = page.getByTestId("examples-card-item");

        this.deleteModal = page.getByTestId("examples-delete-modal");
        this.deleteConfirmBtn = page.getByTestId("examples-delete-modal-confirm");
        this.deleteCancelBtn = page.getByTestId("examples-delete-modal-cancel");
    }

    async goto(): Promise<void> {
        await this.navigate("/examples");
    }

    async expectExamplesVisible(timeout = 30_000): Promise<void> {
        // Leptos WASM hydration may take several seconds after navigation
        await expect(this.examplesPage).toBeVisible({ timeout });
        await expect(this.examplesCard).toBeVisible({ timeout });
        await expect(this.examplesTitle).toBeVisible({ timeout });
    }
}
