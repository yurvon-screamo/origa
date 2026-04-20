import { defineConfig, devices } from "@playwright/test";

const isCI = !!process.env.CI;

export default defineConfig({
    testDir: "./tests",
    timeout: 60000,
    expect: {
        timeout: 10000,
    },
    fullyParallel: true,
    forbidOnly: isCI,
    retries: isCI ? 1 : 0,
    workers: 8,
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
    },
    projects: [
        {
            name: "chromium",
            use: { ...devices["Desktop Chrome"] },
        },
        // {
        //     name: "firefox",
        //     use: { ...devices["Desktop Firefox"] },
        // },
        // Tauri/WebView desktop project placeholder
        // {
        //   name: 'tauri-desktop',
        //   use: { ...devices['Desktop Chrome'] },
        // },
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
});
