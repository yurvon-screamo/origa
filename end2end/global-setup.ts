import type { FullConfig } from "@playwright/test";
import { testUser, trailBaseUrl } from "./config";

const FETCH_TIMEOUT_MS = 30000;

interface GlobalSetupConfig {
	trailBaseUrl: string;
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
 * Try to login with test user to verify it exists
 */
async function verifyTestUserExists(config: GlobalSetupConfig): Promise<boolean> {
	try {
		const response = await fetchWithTimeout(
			`${config.trailBaseUrl}/api/auth/v1/login`,
			{
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					email: testUser.email,
					password: testUser.password,
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
	};
}

export default async function globalSetup(_config: FullConfig): Promise<void> {
	console.log("[global-setup] Starting E2E test setup...");

	const configData = getConfig();

	console.log("[global-setup] Configuration:");
	console.log(`  - TRAILBASE_URL: ${configData.trailBaseUrl}`);
	console.log(`  - TEST_USER_EMAIL: ${testUser.email}`);

	// Set environment variables for tests
	process.env.TRAILBASE_URL = configData.trailBaseUrl;

	// Verify test user exists
	console.log("[global-setup] Verifying test user exists...");
	const userExists = await verifyTestUserExists(configData);

	if (userExists) {
		console.log("[global-setup] ✓ Test user exists and can authenticate");
	} else {
		console.error("\n[global-setup] ❌ TEST USER NOT FOUND OR CANNOT AUTHENTICATE");
		console.error(`\n   The test user '${testUser.email}' does not exist in TrailBase.`);
		console.error(`\n   To fix this, you need to create the test user manually using the TrailBase Admin UI:`);
		console.error(`      1. Go to: ${configData.trailBaseUrl}/_/admin`);
		console.error(`      2. Login as admin`);
		console.error(`      3. Navigate to 'Users' section`);
		console.error(`      4. Create user with email: ${testUser.email}`);
		console.error(`      5. Set password: ${testUser.password}`);
		console.error(`      6. Mark as verified`);
		console.error("\n");
		console.warn("[global-setup] Continuing anyway - tests will fail if user doesn't exist\n");
	}

	console.log("[global-setup] Setup complete.");
}
