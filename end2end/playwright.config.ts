import { defineConfig, devices } from "@playwright/test";

const isCI = !!process.env.CI;

export default defineConfig({
    testDir: "./tests",
    timeout: 120000,
    expect: {
        timeout: 10000,
    },
    fullyParallel: true,
    forbidOnly: isCI,
    retries: isCI ? 2 : 0,
    workers: 1,
    reporter: [
        [
            "html",
            {
                open: "on-failure",
                host: "0.0.0.0",
            },
        ],
    ],
    use: {
        baseURL: "http://localhost:1420",
        trace: "on-first-retry",
        screenshot: "only-on-failure",
        video: "retain-on-failure",
        bypassCSP: true,
    },
    projects: [
        {
            name: "chromium",
            use: {
                ...devices["Desktop Chrome"],
                launchOptions: {
                    args: [
                        "--disable-web-security",
                        "--disable-features=IsolateOrigins,site-per-process",
                        "--disable-site-isolation-trials",
                    ],
                },
            },
        },
    ],
    webServer: {
        command: "cd ../origa_ui && trunk serve",
        url: "http://localhost:1420",
        reuseExistingServer: !isCI,
        timeout: 600000,
        stdout: "pipe",
        stderr: "pipe",
    },
    globalSetup: "./global-setup.ts",
    globalTeardown: "./global-teardown.ts",
});
