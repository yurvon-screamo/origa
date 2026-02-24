import { expect, type Locator, type Page } from "@playwright/test";

export class ProfilePage {
	readonly page: Page;
	readonly heading: Locator;
	readonly usernameInput: Locator;
	readonly levelSelector: Locator;
	readonly languageSelector: Locator;
	readonly duolingoTokenInput: Locator;
	readonly remindersToggle: Locator;
	readonly remindersCheckbox: Locator;
	readonly saveButton: Locator;
	readonly logoutButton: Locator;
	readonly deleteAccountButton: Locator;
	readonly confirmDeleteButton: Locator;
	readonly cancelDeleteButton: Locator;

	constructor(page: Page) {
		this.page = page;
		this.heading = page.getByRole("heading", { name: /Профиль/ });
		this.usernameInput = page
			.getByText("Имя пользователя")
			.locator("..")
			.getByRole("textbox");
		this.levelSelector = page.getByText("Целевой уровень JLPT");
		this.languageSelector = page.getByText("Язык интерфейса");
		this.duolingoTokenInput = page
			.getByText("Duolingo JWT Token")
			.locator("..")
			.getByRole("textbox");
		this.remindersToggle = page.locator(".toggle-container");
		this.remindersCheckbox = page.locator(
			'.toggle-container input[type="checkbox"]',
		);
		this.saveButton = page.getByRole("button", { name: "Сохранить изменения" });
		this.logoutButton = page.getByRole("button", { name: "Выйти из аккаунта" });
		this.deleteAccountButton = page.getByRole("button", {
			name: "Удалить аккаунт",
		});
		this.confirmDeleteButton = page.getByRole("button", {
			name: /Да, удалить аккаунт|Удаление\.\.\./,
		});
		this.cancelDeleteButton = page
			.getByRole("button", { name: "Отмена" })
			.last();
	}

	async expectVisible() {
		await expect(this.heading).toBeVisible();
		await expect(this.usernameInput).toBeVisible();
		await expect(this.levelSelector).toBeVisible();
		await expect(this.languageSelector).toBeVisible();
		await expect(this.duolingoTokenInput).toBeVisible();
		await expect(this.remindersToggle).toBeVisible();
		await expect(this.saveButton).toBeVisible();
		await expect(this.logoutButton).toBeVisible();
		await expect(this.deleteAccountButton).toBeVisible();
	}

	async goto() {
		await this.page.goto("/profile");
	}

	async expectUsername(username: string) {
		await expect(this.usernameInput).toHaveValue(username);
	}

	async setDuolingoToken(token: string) {
		await this.duolingoTokenInput.fill(token);
	}

	async expectDuolingoToken(token: string) {
		await expect(this.duolingoTokenInput).toHaveValue(token);
	}

	async toggleReminders() {
		const checkbox = this.remindersCheckbox;
		const currentState = await checkbox.isChecked();
		
		await this.remindersToggle.click();
		await this.page.waitForTimeout(300);
		
		await expect(checkbox).toBeChecked({ checked: !currentState });
	}

	async expectRemindersEnabled(enabled: boolean) {
		await expect(this.remindersCheckbox).toBeChecked({ checked: enabled });
	}

	async saveChanges() {
		await this.saveButton.click();

		try {
			await expect(this.saveButton).toHaveText("Сохранение...", {
				timeout: 1000,
			});
		} catch (_error) {
			console.log("Loading state may be too fast to capture, continuing...");
		}

		await expect(this.saveButton).toHaveText("Сохранить изменения", {
			timeout: 5000,
		});
	}

	async logout() {
		await this.logoutButton.click();
		await expect(this.page).toHaveURL("/");
	}

	async expectHeadingContains(username: string) {
		await expect(this.heading).toContainText(username);
	}

	async deleteAccount() {
		// Click delete account button to show confirmation
		await this.deleteAccountButton.click();

		// Wait for confirmation dialog
		await expect(this.confirmDeleteButton).toBeVisible();

		// Confirm deletion
		await this.confirmDeleteButton.click();

		// Wait for redirect to login page
		await expect(this.page).toHaveURL("/");
	}

	async cancelDelete() {
		await this.deleteAccountButton.click();
		await expect(this.cancelDeleteButton).toBeVisible();
		await this.cancelDeleteButton.click();
		await expect(this.confirmDeleteButton).not.toBeVisible();
	}
}
