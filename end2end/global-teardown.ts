import type { FullConfig } from "@playwright/test";
import { cleanupOrphanedAccounts } from "./helpers/cleanup";

export default async function globalTeardown(_config: FullConfig): Promise<void> {
	console.log("[global-teardown] Starting cleanup...");

	try {
		await cleanupOrphanedAccounts("global-teardown");
	} catch (error) {
		console.error("[global-teardown] ⚠ Cleanup failed (non-fatal):", error);
	}

	console.log("[global-teardown] Done.");
}
