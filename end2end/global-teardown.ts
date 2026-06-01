import type { FullConfig } from "@playwright/test";
import { cleanupOrphanedAccounts } from "./helpers/cleanup";
import * as fs from "fs";
import * as path from "path";

export default async function globalTeardown(_config: FullConfig): Promise<void> {
    console.log("[global-teardown] Starting cleanup...");

    // Kill TrailBase process if we started it
    const pidFile = path.join(__dirname, ".trailbase.pid");
    if (fs.existsSync(pidFile)) {
        const pid = parseInt(fs.readFileSync(pidFile, "utf-8").trim(), 10);
        if (!isNaN(pid)) {
            try {
                process.kill(pid);
                console.log(`[global-teardown] Killed TrailBase (PID ${pid})`);
            } catch {
                // Process may have already exited
            }
        }
        fs.unlinkSync(pidFile);
    }

    try {
        await cleanupOrphanedAccounts("global-teardown");
    } catch (error) {
        console.error("[global-teardown] ⚠ Cleanup failed (non-fatal):", error);
    }

    console.log("[global-teardown] Done.");
}
