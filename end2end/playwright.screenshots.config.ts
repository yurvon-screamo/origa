import { defineConfig, devices } from "@playwright/test";

// Screenshot-only runner for #528: reuses the project's webServer recipe
// (static cdn + trunk serve with local TrailBase) and a dedicated testDir.
export default defineConfig({
    testDir: "./screenshots-spec",
    timeout: 180_000,
    expect: { timeout: 15_000 },
    workers: 1,
    reporter: [["line"]],
    use: {
        baseURL: "http://localhost:1420",
        trace: "off",
        screenshot: "off",
        ...devices["Desktop Chrome"],
    },
    webServer: [
        {
            command: "npx serve ../cdn -p 8080 --no-clipboard --cors",
            port: 8080,
            reuseExistingServer: true,
            timeout: 30_000,
        },
        {
            command: "cd ../origa_ui && trunk serve",
            port: 1420,
            reuseExistingServer: true,
            timeout: 600_000,
            stdout: "pipe",
            stderr: "pipe",
            env: {
                ORIGA_CDN_BASE_URL: "http://localhost:8080",
                TRAILBASE_URL: "http://127.0.0.1:4000",
            },
        },
    ],
});
