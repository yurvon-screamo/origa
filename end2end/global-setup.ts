import { spawn } from "child_process";
import { end2endRoot } from "./helpers/paths";
import type { FullConfig } from "@playwright/test";
import { fetchWithTimeout } from "./helpers/http";
import { cleanupOrphanedAccountsWithToken } from "./helpers/cleanup";
import { execSync } from "child_process";
import * as fs from "fs";
import * as path from "path";
import { getTrailBaseUrl } from "./config";
import { TRAILBASE_PORT } from "./shots/config.ts";

// Add ~/.local/bin to PATH for trail CLI
if (
  process.platform === "win32" ||
  process.platform === "linux" ||
  process.platform === "darwin"
) {
  const home = process.env.HOME || process.env.USERPROFILE || "";
  if (home) {
    process.env.PATH = `${home}/.local/bin${process.platform === "win32" ? ";" : ":"}${process.env.PATH || ""}`;
  }
}

// TRAILBASE_PORT comes from shots/config.ts (E2E_TRAILBASE_PORT override) —
// imported after ./config so the dotenv bootstrap has already run. The spawn
// address, the healthcheck and the kill must agree on one port, so the
// .env-provided TRAILBASE_URL is re-pinned from the resolved port (dotenv's
// override would otherwise keep a stale port from a shared checkout copy).
process.env.TRAILBASE_URL = `http://127.0.0.1:${TRAILBASE_PORT}`;

async function waitForTrailBase(
  maxRetries = 30,
  intervalMs = 1000,
): Promise<boolean> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await fetchWithTimeout(
        `${getTrailBaseUrl()}/api/healthcheck`,
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

async function getAdminToken(): Promise<{ token: string; csrfToken: string }> {
  const adminEmail = process.env.ORIGA_ADMIN_EMAIL || "admin@localhost";
  const adminPassword = process.env.ORIGA_ADMIN_PASSWORD;

  if (!adminPassword) {
    throw new Error(
      "ORIGA_ADMIN_PASSWORD is not set. Configure it in .env file.",
    );
  }

  const response = await fetchWithTimeout(
    `${getTrailBaseUrl()}/api/auth/v1/login`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        email: adminEmail,
        password: adminPassword,
      }),
    },
  );

  if (!response.ok) {
    throw new Error(`Admin login failed: ${response.status}`);
  }

  const data = (await response.json()) as {
    auth_token: string;
    csrf_token: string;
  };
  if (!data.csrf_token) {
    throw new Error("Admin login response missing csrf_token");
  }

  return { token: data.auth_token, csrfToken: data.csrf_token };
}

function setAdminPassword(): void {
  try {
    execSync(
      `trail user change-password ${process.env.ORIGA_ADMIN_EMAIL || "admin@localhost"} ${process.env.ORIGA_ADMIN_PASSWORD}`,
      {
        cwd: `${end2endRoot}/trailbase-fixture`,
        stdio: "pipe",
        timeout: 10000,
      },
    );
    console.log("[global-setup] ✓ Admin password set to known value");
  } catch (error) {
    console.error("[global-setup] ⚠ Failed to change admin password:", error);
  }
}

function killExistingTrailBase(): void {
  try {
    if (process.platform === "win32") {
      execSync("taskkill /f /im trail.exe", { stdio: "pipe" });
    } else {
      // Port-scoped pattern: a parallel checkout on a shared machine runs
      // its own TrailBase on a different port (E2E_TRAILBASE_PORT); a bare
      // `pkill -f 'trail run'` would take that stranger down too.
      execSync(
        `pkill -f 'trail run --dev --address 127.0.0.1:${TRAILBASE_PORT}'`,
        { stdio: "pipe" },
      );
    }
    console.log("[global-setup] ✓ Killed existing TrailBase process");
  } catch {
    // No process found — that's fine
  }
}

async function startTrailBase(): Promise<void> {
  console.log("[global-setup] Starting local TrailBase...");
  console.log(`  - URL: ${getTrailBaseUrl()}`);

  const args = ["run", "--dev", "--address", `127.0.0.1:${TRAILBASE_PORT}`];

  const proc = spawn("trail", args, {
    cwd: `${end2endRoot}/trailbase-fixture`,
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

  // Save PID for cleanup in teardown
  fs.writeFileSync(path.join(end2endRoot, ".trailbase.pid"), String(proc.pid));

  console.log("[global-setup] Waiting for TrailBase to be ready...");
  const ready = await waitForTrailBase();
  if (!ready) {
    throw new Error("TrailBase failed to start within timeout");
  }
  console.log("[global-setup] ✓ TrailBase is ready");
}

export default async function globalSetup(_: FullConfig): Promise<void> {
  console.log("[global-setup] Starting E2E test setup...");

  // Always kill existing TrailBase and start fresh to avoid stale state
  killExistingTrailBase();
  setAdminPassword();
  await startTrailBase();

  // Login as admin — single login reused for verification + cleanup
  console.log("[global-setup] Logging in as admin...");
  const adminAuth = await getAdminToken();
  console.log("[global-setup] ✓ Admin credentials verified");

  // Cleanup orphaned test accounts — reuse admin token (no second login)
  console.log("[global-setup] Cleaning up orphaned test accounts...");
  try {
    await cleanupOrphanedAccountsWithToken(
      "global-setup",
      adminAuth.token,
      adminAuth.csrfToken,
    );
  } catch (error) {
    console.error("[global-setup] ⚠ Cleanup failed (non-fatal):", error);
  }

  console.log("[global-setup] Setup complete.");
}
