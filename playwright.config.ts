import path from "node:path";
import { defineConfig, devices } from "@playwright/test";
import dotenv from "dotenv";

dotenv.config({ path: path.resolve(__dirname, "e2e", ".env") });

export default defineConfig({
	testDir: "./e2e",
	fullyParallel: false,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 0,
	workers: 2,
	timeout: 60000,
	reporter: "html",
	use: {
		baseURL: "http://localhost:8080",
		trace: "on-first-retry",
		screenshot: "only-on-failure",
	},

	projects: [
		{
			name: "chromium",
			use: { ...devices["Desktop Chrome"] },
		},
	],

	webServer: {
		command: "cd origa_ui && trunk serve --port 8080",
		url: "http://localhost:8080",
		reuseExistingServer: true,
		timeout: 120 * 1000,
	},
});
