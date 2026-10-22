import { Page } from "@playwright/test";
import { CardListPage } from "./card_list.page";

export class ExamplesPage extends CardListPage {
    constructor(page: Page) {
        super(page, "examples");
    }

    async goto(): Promise<void> {
        await this.navigate("/examples");
    }
}
