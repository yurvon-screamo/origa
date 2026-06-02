import type { FullConfig } from "@playwright/test";
import * as fs from "fs";
import * as path from "path";

export default async function globalTeardown(_config: FullConfig): Promise<void> {
    console.log("[global-teardown] Starting teardown...");

    // Kill TrailBase process
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

    console.log("[global-teardown] Done.");
}
