import { spawn, type ChildProcess } from "child_process";
import type { FullConfig } from "@playwright/test";
import { fetchWithTimeout } from "./helpers/http";
import { cleanupOrphanedAccounts } from "./helpers/cleanup";
import { execSync } from "child_process";

// Add ~/.local/bin to PATH for trail CLI
if (process.platform === "win32" || process.platform === "linux" || process.platform === "darwin") {
    const home = process.env.HOME || process.env.USERPROFILE || "";
    if (home) {
        process.env.PATH = `${home}/.local/bin${process.platform === "win32" ? ";" : ":"}${process.env.PATH || ""}`;
    }
}

const TRAILBASE_PORT = 4000;
const TRAILBASE_URL = `http://127.0.0.1:${TRAILBASE_PORT}`;
const ADMIN_EMAIL = "admin@localhost";
const ADMIN_PASSWORD = "secret";

async function waitForTrailBase(maxRetries = 30, intervalMs = 1000): Promise<boolean> {
    for (let i = 0; i < maxRetries; i++) {
        try {
            const response = await fetchWithTimeout(
                `${TRAILBASE_URL}/api/healthcheck`,
                {},
                5000,
            );
            if (response.ok) {
                return true;
            }
        } catch {
            // not ready yet
        }
        await new Promise((resolve) => setTimeout(resolve, intervalMs));
    }
    return false;
}

async function isTrailBaseRunning(): Promise<boolean> {
    try {
        const response = await fetchWithTimeout(`${TRAILBASE_URL}/api/healthcheck`, {}, 3000);
        return response.ok;
    } catch {
        return false;
    }
}

async function verifyAdminCredentials(): Promise<boolean> {
    try {
        const response = await fetchWithTimeout(
            `${TRAILBASE_URL}/api/auth/v1/login`,
            {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    email: ADMIN_EMAIL,
                    password: ADMIN_PASSWORD,
                }),
            },
        );
        return response.ok;
    } catch {
        return false;
    }
}

function setAdminPassword(): void {
    try {
        execSync(
            `trail user change-password ${ADMIN_EMAIL} ${ADMIN_PASSWORD}`,
            {
                cwd: `${__dirname}/trailbase-fixture`,
                stdio: "pipe",
                timeout: 10000,
            },
        );
        console.log("[global-setup] ✓ Admin password set to known value");
    } catch (error) {
        console.error("[global-setup] ⚠ Failed to change admin password:", error);
    }
}

async function startTrailBase(): Promise<void> {
    console.log("[global-setup] Starting local TrailBase...");
    console.log(`  - URL: ${TRAILBASE_URL}`);

    const args = [
        "run",
        "--dev",
        "--address", `127.0.0.1:${TRAILBASE_PORT}`,
    ];

    const proc = spawn("trail", args, {
        cwd: `${__dirname}/trailbase-fixture`,
        stdio: ["ignore", "pipe", "pipe"],
        detached: true,
    });

    proc.stdout?.on("data", (data: Buffer) => {
        for (const line of data.toString().trim().split("\n")) {
            console.log(`[trailbase] ${line}`);
        }
    });

    proc.stderr?.on("data", (data: Buffer) => {
        for (const line of data.toString().trim().split("\n")) {
            console.error(`[trailbase] ${line}`);
        }
    });

    // Don't let parent process wait for child
    proc.unref();

    console.log("[global-setup] Waiting for TrailBase to be ready...");
    const ready = await waitForTrailBase();
    if (!ready) {
        throw new Error("TrailBase failed to start within timeout");
    }
    console.log("[global-setup] ✓ TrailBase is ready");
}

export default async function globalSetup(_config: FullConfig): Promise<void> {
    console.log("[global-setup] Starting E2E test setup...");

    // Set environment variables for tests
    process.env.TRAILBASE_URL = TRAILBASE_URL;
    process.env.ORIGA_ADMIN_EMAIL = ADMIN_EMAIL;
    process.env.ORIGA_ADMIN_PASSWORD = ADMIN_PASSWORD;

    // Start local TrailBase if not already running
    const running = await isTrailBaseRunning();
    if (running) {
        console.log("[global-setup] ✓ TrailBase already running");
    } else {
        await startTrailBase();
        // Set known admin password after first start
        setAdminPassword();
    }

    // Verify admin credentials
    console.log("[global-setup] Verifying admin credentials...");
    const adminOk = await verifyAdminCredentials();

    if (adminOk) {
        console.log("[global-setup] ✓ Admin credentials verified");
    } else {
        throw new Error("Admin credentials verification failed. Check TrailBase setup.");
    }

    // Cleanup orphaned test accounts
    console.log("[global-setup] Cleaning up orphaned test accounts...");
    try {
        await cleanupOrphanedAccounts("global-setup");
    } catch (error) {
        console.error("[global-setup] ⚠ Cleanup failed (non-fatal):", error);
    }

    console.log("[global-setup] Setup complete.");
}
