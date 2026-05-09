import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class ProfilePage extends BasePage {
	readonly profilePage: Locator;
	readonly profileCard: Locator;
	readonly profileContent: Locator;

	readonly profilePersonalData: Locator;
	readonly profileSettings: Locator;
	readonly profileActions: Locator;

	readonly langEnglish: Locator;
	readonly langRussian: Locator;

	readonly loadMinimal: Locator;
	readonly loadLight: Locator;
	readonly loadMedium: Locator;
	readonly loadHard: Locator;
	readonly loadImpossible: Locator;
	readonly loadInsane: Locator;

	readonly confirmDeleteBtn: Locator;
	readonly cancelDeleteBtn: Locator;

	constructor(page: Page) {
		super(page);

		this.profilePage = page.getByTestId("profile-page");
		this.profileCard = page.getByTestId("profile-card");
		this.profileContent = page.getByTestId("profile-content");

		this.profilePersonalData = page.getByTestId("profile-personal-data");
		this.profileSettings = page.getByTestId("profile-settings");
		this.profileActions = page.getByTestId("profile-actions");

		this.langEnglish = page.getByTestId("profile-lang-english");
		this.langRussian = page.getByTestId("profile-lang-russian");

		this.loadMinimal = page.getByTestId("profile-load-minimal");
		this.loadLight = page.getByTestId("profile-load-light");
		this.loadMedium = page.getByTestId("profile-load-medium");
		this.loadHard = page.getByTestId("profile-load-hard");
		this.loadImpossible = page.getByTestId("profile-load-impossible");
		this.loadInsane = page.getByTestId("profile-load-insane");

		this.confirmDeleteBtn = page.getByTestId("profile-confirm-delete-btn");
		this.cancelDeleteBtn = page.getByTestId("profile-cancel-delete-btn");
	}

	async goto(): Promise<void> {
		await this.navigate("/profile");
	}

	async expectProfileVisible(): Promise<void> {
		await expect(this.profilePage).toBeVisible();
		await expect(this.profileCard).toBeVisible();
		await expect(this.profileContent).toBeVisible();
	}

	async selectLanguage(lang: "english" | "russian"): Promise<void> {
		const btn = lang === "english" ? this.langEnglish : this.langRussian;
		await btn.click();
	}

	async selectDailyLoad(load: "minimal" | "light" | "medium" | "hard" | "impossible" | "insane"): Promise<void> {
		const btns: Record<string, Locator> = {
			minimal: this.loadMinimal,
			light: this.loadLight,
			medium: this.loadMedium,
			hard: this.loadHard,
			impossible: this.loadImpossible,
			insane: this.loadInsane,
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
		await this.navigateToHomeAndWaitForSync();
	}

	async navigateToHomeAndWaitForSync(timeout = 15_000): Promise<void> {
		await this.page.goto("/home", { waitUntil: "domcontentloaded" });

		const successToast = this.page
			.locator('[data-testid="home-toasts"]')
			.locator("div.toast-success");
		try {
			await successToast.waitFor({ state: "visible", timeout });
		} catch {
			// Sync may have completed before page loaded
		}

		await this.page.goto("/profile", { waitUntil: "domcontentloaded" });
		await this.page.getByTestId("profile-page").waitFor({ state: "visible", timeout: 15_000 });
	}

	async waitForSaveComplete(): Promise<void> {
		await expect(this.page.getByTestId("profile-save-btn")).toBeEnabled({ timeout: 10_000 });
	}
}
