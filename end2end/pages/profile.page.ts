import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class ProfilePage extends BasePage {
	readonly profilePage: Locator;
	readonly profileCard: Locator;
	readonly profileTitle: Locator;
	readonly profileBackBtn: Locator;
	readonly profileContent: Locator;

	readonly profilePersonalData: Locator;
	readonly profileSettings: Locator;
	readonly profileActions: Locator;

	readonly langEnglish: Locator;
	readonly langRussian: Locator;

	readonly loadLight: Locator;
	readonly loadMedium: Locator;
	readonly loadHard: Locator;
	readonly loadImpossible: Locator;

	readonly confirmDeleteBtn: Locator;
	readonly cancelDeleteBtn: Locator;

	constructor(page: Page) {
		super(page);

		this.profilePage = page.getByTestId("profile-page");
		this.profileCard = page.getByTestId("profile-card");
		this.profileTitle = page.getByTestId("profile-title");
		this.profileBackBtn = page.getByTestId("profile-back-btn");
		this.profileContent = page.getByTestId("profile-content");

		this.profilePersonalData = page.getByTestId("profile-personal-data");
		this.profileSettings = page.getByTestId("profile-settings");
		this.profileActions = page.getByTestId("profile-actions");

		this.langEnglish = page.getByTestId("profile-lang-english");
		this.langRussian = page.getByTestId("profile-lang-russian");

		this.loadLight = page.getByTestId("profile-load-light");
		this.loadMedium = page.getByTestId("profile-load-medium");
		this.loadHard = page.getByTestId("profile-load-hard");
		this.loadImpossible = page.getByTestId("profile-load-impossible");

		this.confirmDeleteBtn = page.getByTestId("profile-confirm-delete-btn");
		this.cancelDeleteBtn = page.getByTestId("profile-cancel-delete-btn");
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

	async selectLanguage(lang: "english" | "russian"): Promise<void> {
		const btn = lang === "english" ? this.langEnglish : this.langRussian;
		await btn.click();
	}

	async selectDailyLoad(load: "light" | "medium" | "hard" | "impossible"): Promise<void> {
		const btns: Record<string, Locator> = {
			light: this.loadLight,
			medium: this.loadMedium,
			hard: this.loadHard,
			impossible: this.loadImpossible,
		};
		await btns[load].click();
	}

	async saveProfile(): Promise<void> {
		await this.page.getByTestId("profile-save-btn").click();
	}

	async deleteAccount(): Promise<void> {
		await this.page.getByTestId("profile-delete-btn").click();
	}

	async confirmDelete(): Promise<void> {
		await this.confirmDeleteBtn.click();
	}

	async cancelDelete(): Promise<void> {
		await this.cancelDeleteBtn.click();
	}

	async navigateToHomeAndBack(): Promise<void> {
		await this.page.goto("/home");
		await this.page.getByTestId("home-content").waitFor({ state: "visible" });
		await this.goto();
		await this.expectProfileVisible();
	}

	async navigateToHomeAndWaitForSync(timeout = 15_000): Promise<void> {
		await this.page.goto("/home");
		await this.page.getByTestId("home-content").waitFor({ state: "visible" });

		const successToast = this.page
			.locator('[data-testid="home-toasts"]')
			.locator("div.toast-success");
		try {
			await successToast.waitFor({ state: "visible", timeout });
		} catch {
			await this.page.waitForTimeout(3000);
		}

		await this.goto();
	}

	async waitForSaveComplete(): Promise<void> {
		await expect(this.page.getByTestId("profile-save-btn")).toBeEnabled({ timeout: 10_000 });
	}
}
