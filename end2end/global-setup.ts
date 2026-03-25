import type { FullConfig } from "@playwright/test";
import { trailBaseUrl } from "./config";

const FETCH_TIMEOUT_MS = 30000;

interface GlobalSetupConfig {
	trailBaseUrl: string;
	adminEmail: string;
	adminPassword: string;
}

async function fetchWithTimeout(
	url: string,
	options: RequestInit,
	timeout: number = FETCH_TIMEOUT_MS,
): Promise<Response> {
	const controller = new AbortController();
	const timeoutId = setTimeout(() => controller.abort(), timeout);

	try {
		const response = await fetch(url, {
			...options,
			signal: controller.signal,
		});
		return response;
	} finally {
		clearTimeout(timeoutId);
	}
}

/**
 * Verify admin credentials work for creating test users
 */
async function verifyAdminCredentials(config: GlobalSetupConfig): Promise<boolean> {
	try {
		const response = await fetchWithTimeout(
			`${config.trailBaseUrl}/api/auth/v1/login`,
			{
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					email: config.adminEmail,
					password: config.adminPassword,
				}),
			},
		);
		return response.ok;
	} catch {
		return false;
	}
}

function getConfig(): GlobalSetupConfig {
	return {
		trailBaseUrl: process.env.TRAILBASE_URL || "https://origa.uwuwu.net",
		adminEmail: process.env.ORIGA_ADMIN_EMAIL || "admin@localhost",
		adminPassword: process.env.ORIGA_ADMIN_PASSWORD || "",
	};
}

export default async function globalSetup(_config: FullConfig): Promise<void> {
	console.log("[global-setup] Starting E2E test setup...");

	const configData = getConfig();

	console.log("[global-setup] Configuration:");
	console.log(`  - TRAILBASE_URL: ${configData.trailBaseUrl}`);
	console.log(`  - ADMIN_EMAIL: ${configData.adminEmail}`);

	// Set environment variables for tests
	process.env.TRAILBASE_URL = configData.trailBaseUrl;

	// Verify admin credentials
	console.log("[global-setup] Verifying admin credentials...");
	const adminOk = await verifyAdminCredentials(configData);

	if (adminOk) {
		console.log("[global-setup] ✓ Admin credentials verified - can create test users");
	} else {
		console.error("\n[global-setup] ❌ ADMIN CREDENTIALS INVALID OR MISSING");
		console.error("\n   Test users are created dynamically via fixtures.");
		console.error("   Admin credentials are required to create/delete test users.");
		console.error("\n   Set environment variables:");
		console.error("      ORIGA_ADMIN_EMAIL=admin@example.com");
		console.error("      ORIGA_ADMIN_PASSWORD=your-password");
		console.error("\n   Or create .env file in end2end/ directory:");
		console.error("      ORIGA_ADMIN_EMAIL=admin@localhost");
		console.error("      ORIGA_ADMIN_PASSWORD=your-admin-password");
		console.error("\n");
		throw new Error("Admin credentials required for E2E tests");
	}

	console.log("[global-setup] Setup complete.");
}
