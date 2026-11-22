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
import { winTrailKillCommand } from "./trail-kill.ts";

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

// TRAILBASE_PORT comes from shots/config.ts (E2E_TRAILBASE_PORT override).
// Module order does NOT make the .env visible here: shots/config.ts is a
// dependency of ./config, so it is evaluated before config.ts's dotenv
// bootstrap either way — the ports resolve from shell env only (the
// contract documented in end2end/AGENTS.md). The trail process spawns on
// 127.0.0.1:PORT and the healthcheck waits on that exact address
// (waitForTrailBase uses trailbaseSpawnUrl below). This setup reads no
// TRAILBASE_URL: the bundle gets its compile-time URL from the webServer
// env, and the offline intercept steps derive both host forms from the
// port (getTrailBaseUrl), so there is nothing shell-level to preserve.

/** The address the trail process is spawned on (loopback, always reachable). */
const trailbaseSpawnUrl = `http://127.0.0.1:${TRAILBASE_PORT}`;

async function waitForTrailBase(
  maxRetries = 30,
  intervalMs = 1000,
): Promise<boolean> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await fetchWithTimeout(
        `${trailbaseSpawnUrl}/api/healthcheck`,
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
      // Port-scoped via winTrailKillCommand() (trail-kill.ts): the
      // command line carries the exact spawned address, so a parallel
      // checkout's TrailBase on another E2E_TRAILBASE_PORT survives.
      execSync(winTrailKillCommand(TRAILBASE_PORT), { stdio: "pipe" });
    } else {
      // Port-scoped pattern, anchored at the end of the address argument
      // (ERE, ( |$)): a bare substring would let a configured 4000 match
      // a stranger's 40000. A parallel checkout on a shared machine runs
      // its own TrailBase on a different port (E2E_TRAILBASE_PORT); a
      // bare `pkill -f 'trail run'` would take that stranger down too.
      execSync(
        `pkill -f 'trail run --dev --address 127\\.0\\.0\\.1:${TRAILBASE_PORT}( |$)'`,
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
