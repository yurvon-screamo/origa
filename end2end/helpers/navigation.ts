import { expect, type Page } from "@playwright/test";

export async function skipOnboarding(page: Page): Promise<void> {
	await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({
		timeout: 10000,
	});
	const skipButton = page.getByTestId("onboarding-skip");
	if (await skipButton.isVisible().catch(() => false)) {
		await skipButton.click();
	}
	await page.waitForURL(/\/home$/, { timeout: 10000 });
}
