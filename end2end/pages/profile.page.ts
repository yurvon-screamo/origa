import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class ProfilePage extends BasePage {
	// Page structure
	readonly profilePage: Locator;
	readonly profileCard: Locator;
	readonly profileTitle: Locator;
	readonly profileBackBtn: Locator;
	readonly profileContent: Locator;

	// Content sections
	readonly profilePersonalData: Locator;
	readonly profileSettings: Locator;
	readonly profileActions: Locator;

	constructor(page: Page) {
		super(page);

		// Page structure
		this.profilePage = page.getByTestId("profile-page");
		this.profileCard = page.getByTestId("profile-card");
		this.profileTitle = page.getByTestId("profile-title");
		this.profileBackBtn = page.getByTestId("profile-back-btn");
		this.profileContent = page.getByTestId("profile-content");

		// Content sections
		this.profilePersonalData = page.getByTestId("profile-personal-data");
		this.profileSettings = page.getByTestId("profile-settings");
		this.profileActions = page.getByTestId("profile-actions");
	}

	async goto(): Promise<void> {
		await this.navigate("/profile");
	}

	async expectProfileVisible(): Promise<void> {
		await expect(this.profilePage).toBeVisible();
		await expect(this.profileCard).toBeVisible();
		await expect(this.profileTitle).toBeVisible();
		await expect(this.profileContent).toBeVisible();
	}

	async clickBack(): Promise<void> {
		await this.profileBackBtn.click();
	}
}