import { test, expect } from "@playwright/test";
import { skipInCI } from "./setup";

test.describe("Tauri Desktop Tests", () => {
	test.skip(skipInCI, "Tauri tests require desktop build");

	test("should open Tauri window", async ({ page }) => {
		// This test requires Tauri desktop app to be running
		// Run manually: cd tauri && cargo tauri dev
		// Then: npx playwright test tests/tauri/desktop.spec.ts --headed

		test.skip();

		// When running Tauri:
		// await page.goto('tauri://localhost');
		// await expect(page.locator('body')).toBeVisible();
	});
});
