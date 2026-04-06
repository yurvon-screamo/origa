import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

type GrammarLevel = "N5" | "N4" | "N3" | "N2" | "N1";
type FilterType = "Все" | "Новые" | "Сложные" | "В процессе" | "Изученные";

export class GrammarPage extends BasePage {
    readonly grammarPage: Locator;
    readonly grammarCard: Locator;
    readonly grammarTitle: Locator;
    readonly backBtn: Locator;
    readonly addBtn: Locator;
    readonly searchInput: Locator;
    readonly grammarGrid: Locator;
    readonly emptyState: Locator;

    readonly drawer: Locator;
    readonly drawerAddBtn: Locator;
    readonly drawerSelectAllBtn: Locator;
    readonly drawerSearchInput: Locator;

    readonly deleteModal: Locator;
    readonly deleteConfirmBtn: Locator;
    readonly deleteCancelBtn: Locator;

    constructor(page: Page) {
        super(page);

        this.grammarPage = page.getByTestId("grammar-page");
        this.grammarCard = page.getByTestId("grammar-card");
        this.grammarTitle = page.getByTestId("grammar-title");
        this.backBtn = page.getByTestId("grammar-back-btn");
        this.addBtn = page.getByTestId("grammar-add-btn");
        this.searchInput = page.getByTestId("grammar-search-input");
        this.grammarGrid = page.getByTestId("grammar-grid");
        this.emptyState = page.getByTestId("grammar-empty-state");

        this.drawer = page.getByTestId("grammar-add-drawer");
        this.drawerAddBtn = page.getByTestId("grammar-drawer-add-btn");
        this.drawerSelectAllBtn = page.getByTestId("grammar-drawer-select-all-btn");
        this.drawerSearchInput = page.getByTestId("grammar-drawer-search-input");

        this.deleteModal = page.getByTestId("grammar-delete-modal");
        this.deleteConfirmBtn = page.getByTestId("grammar-delete-modal-confirm");
        this.deleteCancelBtn = page.getByTestId("grammar-delete-modal-cancel");
    }

    async goto(): Promise<void> {
        await this.navigate("/grammar");
    }

    async expectGrammarVisible(): Promise<void> {
        await expect(this.grammarPage).toBeVisible();
        await expect(this.grammarCard).toBeVisible();
        await expect(this.grammarTitle).toBeVisible();
    }

    async searchGrammar(query: string): Promise<void> {
        await this.searchInput.waitFor({ state: "visible", timeout: 5000 });
        await this.searchInput.fill(query);
    }

    async clickBack(): Promise<void> {
        await this.backBtn.click();
    }

    async openAddModal(): Promise<void> {
        await this.addBtn.click();
        await expect(this.drawer).toBeVisible({ timeout: 5000 });
    }

    async selectLevel(level: GrammarLevel): Promise<void> {
        await this.page.getByTestId("grammar-level-" + level.toLowerCase()).click();
        const ruleItem = this.drawer.getByTestId("grammar-drawer-item").first();
        const emptyMsg = this.drawer.getByTestId("grammar-drawer-empty");
        await expect(ruleItem.or(emptyMsg)).toBeVisible({ timeout: 10_000 });
    }

    async selectRule(title: string): Promise<void> {
        const rule = this.drawer.getByTestId("grammar-drawer-item").filter({ hasText: title });
        await expect(rule.first()).toBeVisible({ timeout: 5000 });
        await rule.first().click();
    }

    async selectAllRules(): Promise<void> {
        await this.drawerSelectAllBtn.click();
    }

    async addSelectedRules(): Promise<void> {
        await this.drawerAddBtn.click({ timeout: 5000 });
        await expect(this.drawer).not.toBeVisible({ timeout: 15_000 });
    }

    async selectFilter(name: FilterType): Promise<void> {
        const filterMap: Record<FilterType, string> = {
            "Все": "all",
            "Новые": "new",
            "Сложные": "hard",
            "В процессе": "in-progress",
            "Изученные": "learned",
        };
        await this.page.getByTestId("grammar-filter-" + filterMap[name]).click();
    }

    async getCardCount(): Promise<number> {
        return this.page.getByTestId("grammar-card-item").count();
    }

    async deleteCardByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("grammar-card-item").nth(index);
        await card.getByTestId("grammar-card-item-delete-btn").click();
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteConfirmBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 10_000 });
    }

    async cancelDeleteCardByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("grammar-card-item").nth(index);
        await card.getByTestId("grammar-card-item-delete-btn").click();
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteCancelBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 5000 });
    }

    async markCardAsKnownByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("grammar-card-item").nth(index);
        await card.getByTestId("grammar-card-item-mark-known-btn").click();
        await this.page.waitForTimeout(500);
    }
}
