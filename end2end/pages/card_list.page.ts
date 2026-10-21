import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export const CARD_LIST_VISIBLE_TIMEOUT = 30_000;

/**
 * Common locators and assertions of the card list pages (phrases,
 * examples): the shared `card_list_view` skeleton — page/card/title,
 * search, filter grid, empty state, load-more, item and delete modal.
 * A concrete page passes its test-id prefix; page-specific locators and
 * flows stay in the subclass (#823).
 */
export class CardListPage extends BasePage {
    readonly pageRoot: Locator;
    readonly card: Locator;
    readonly title: Locator;
    readonly searchInput: Locator;
    readonly grid: Locator;
    readonly emptyState: Locator;
    readonly loadMoreButton: Locator;
    readonly cardItem: Locator;
    readonly deleteModal: Locator;
    readonly deleteConfirmBtn: Locator;
    readonly deleteCancelBtn: Locator;

    constructor(page: Page, prefix: string) {
        super(page);
        this.pageRoot = page.getByTestId(`${prefix}-page`);
        this.card = page.getByTestId(`${prefix}-card`);
        this.title = page.getByTestId(`${prefix}-title`);
        this.searchInput = page.getByTestId(`${prefix}-search-input`);
        this.grid = page.getByTestId(`${prefix}-grid`);
        this.emptyState = page.getByTestId(`${prefix}-empty-state`);
        this.loadMoreButton = page.getByTestId(`${prefix}-load-more-btn`);
        this.cardItem = page.getByTestId(`${prefix}-card-item`);
        this.deleteModal = page.getByTestId(`${prefix}-delete-modal`);
        this.deleteConfirmBtn = page.getByTestId(`${prefix}-delete-modal-confirm`);
        this.deleteCancelBtn = page.getByTestId(`${prefix}-delete-modal-cancel`);
    }

    async expectVisible(timeout = CARD_LIST_VISIBLE_TIMEOUT): Promise<void> {
        // Leptos WASM hydration may take several seconds after navigation
        await expect(this.pageRoot).toBeVisible({ timeout });
        await expect(this.card).toBeVisible({ timeout });
        await expect(this.title).toBeVisible({ timeout });
    }
}
