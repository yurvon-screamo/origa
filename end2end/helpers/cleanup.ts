import { getAdminToken, listAllTestUsers, deleteTestUserWithRetry } from "../fixtures/admin";
import { trailBaseUrl } from "../config";
import { fetchWithTimeout } from "./http";

const CLEANUP_BATCH_SIZE = 20;

export async function cleanupOrphanedAccounts(prefix: string): Promise<void> {
	const adminAuth = await getAdminToken();
	const orphanedUsers = await listAllTestUsers(adminAuth.token, adminAuth.csrfToken);

	if (orphanedUsers.length === 0) {
		console.log(`[${prefix}] ✓ No orphaned accounts found`);
	} else {
		console.log(`[${prefix}] Found ${orphanedUsers.length} orphaned accounts, deleting...`);

		let succeeded = 0;
		let failed = 0;

		for (let i = 0; i < orphanedUsers.length; i += CLEANUP_BATCH_SIZE) {
			const batch = orphanedUsers.slice(i, i + CLEANUP_BATCH_SIZE);
			const results = await Promise.allSettled(
				batch.map((user) =>
					deleteTestUserWithRetry(adminAuth.token, adminAuth.csrfToken, user.id, user.email),
				),
			);
			succeeded += results.filter((r) => r.status === "fulfilled").length;
			failed += results.filter((r) => r.status === "rejected").length;
		}

		console.log(`[${prefix}] ✓ Cleanup complete: ${succeeded} deleted, ${failed} failed`);
	}

	await cleanupOrphanedUserRecords(prefix, adminAuth.token, adminAuth.csrfToken);
}

async function cleanupOrphanedUserRecords(
	prefix: string,
	token: string,
	csrfToken: string,
): Promise<void> {
	const countResponse = await fetchWithTimeout(
		`${trailBaseUrl}/api/_admin/query`,
		{
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				Authorization: `Bearer ${token}`,
				"csrf-token": csrfToken,
			},
			body: JSON.stringify({
				query: "SELECT COUNT(*) as cnt FROM user WHERE email LIKE 'e2e-%'",
			}),
		},
		10000,
	);

	if (!countResponse.ok) {
		console.error(`[${prefix}] ⚠ Failed to count orphaned user records: ${countResponse.status}`);
		return;
	}

	const countData = (await countResponse.json()) as { rows: [[{ Integer: number }]] };
	const count = countData.rows?.[0]?.[0]?.Integer ?? 0;

	if (count === 0) {
		console.log(`[${prefix}] ✓ No orphaned user records found`);
		return;
	}

	console.log(`[${prefix}] Found ${count} orphaned user records, deleting...`);

	const deleteResponse = await fetchWithTimeout(
		`${trailBaseUrl}/api/_admin/query`,
		{
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				Authorization: `Bearer ${token}`,
				"csrf-token": csrfToken,
			},
			body: JSON.stringify({
				query: "DELETE FROM user WHERE email LIKE 'e2e-%'",
			}),
		},
		30000,
	);

	if (!deleteResponse.ok) {
		console.error(`[${prefix}] ⚠ Failed to delete orphaned user records: ${deleteResponse.status}`);
		return;
	}

	console.log(`[${prefix}] ✓ Deleted ${count} orphaned user records`);
}
