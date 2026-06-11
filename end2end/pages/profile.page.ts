import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class ProfilePage extends BasePage {
	readonly profilePage: Locator;
	readonly profileCard: Locator;
	readonly profileContent: Locator;

	readonly profilePersonalData: Locator;
	readonly profileSettings: Locator;
	readonly profileDangerZone: Locator;
	readonly autosaveStatus: Locator;

	readonly langEnglish: Locator;
	readonly langRussian: Locator;

	readonly loadMinimal: Locator;
	readonly loadLight: Locator;
	readonly loadMedium: Locator;
	readonly loadHard: Locator;
	readonly loadHeavy: Locator;
	readonly loadMaximum: Locator;

	readonly confirmDeleteBtn: Locator;
	readonly cancelDeleteBtn: Locator;

	constructor(page: Page) {
		super(page);

		this.profilePage = page.getByTestId("profile-page");
		this.profileCard = page.getByTestId("profile-card");
		this.profileContent = page.getByTestId("profile-content");

		this.profilePersonalData = page.getByTestId("profile-personal-data");
		this.profileSettings = page.getByTestId("profile-settings");
		this.profileDangerZone = page.getByTestId("profile-danger-zone");
		this.autosaveStatus = page.getByTestId("profile-autosave-status");

		this.langEnglish = page.getByTestId("lang-toggle-en");
		this.langRussian = page.getByTestId("lang-toggle-ru");

		this.loadMinimal = page.getByTestId("profile-load-minimal");
		this.loadLight = page.getByTestId("profile-load-light");
		this.loadMedium = page.getByTestId("profile-load-medium");
		this.loadHard = page.getByTestId("profile-load-hard");
		this.loadHeavy = page.getByTestId("profile-load-heavy");
		this.loadMaximum = page.getByTestId("profile-load-maximum");

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

	async selectDailyLoad(load: "minimal" | "light" | "medium" | "hard" | "heavy" | "maximum"): Promise<void> {
		const btns: Record<string, Locator> = {
			minimal: this.loadMinimal,
			light: this.loadLight,
			medium: this.loadMedium,
			hard: this.loadHard,
			heavy: this.loadHeavy,
			maximum: this.loadMaximum,
		};
		await btns[load].click();
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

	async waitForAutoSave(): Promise<void> {
		const status = this.autosaveStatus;
		// Wait for status to appear (Saving state)
		await status.waitFor({ state: "visible", timeout: 5_000 });
		// Wait for it to show "Saved" or disappear (Idle after fade)
		await expect(status).toContainText(/saved|сохранено/i, { timeout: 10_000 });
	}
}
