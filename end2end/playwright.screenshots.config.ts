import { defineConfig, devices } from "@playwright/test";

const isCI = !!process.env.CI;

// Screenshot-only runner for #528: reuses the project's webServer recipe
// (static cdn + the app server) and a dedicated testDir.
//
// TrailBase comes from global-setup.ts, the same file the BDD suite uses
// (see end2end/AGENTS.md). The specs log in as admin@localhost, and
// TrailBase creates that account with a RANDOM password on a fresh depot
// — global-setup.ts is what sets the known one. The trail webServer entry
// that used to live here started TrailBase in the config directory instead
// of end2end/trailbase-fixture, so the instance had no `user` /
// `domain_user` record APIs and the app could not read its own profile.
export default defineConfig({
    testDir: "./screenshots-spec",
    timeout: 180_000,
    expect: { timeout: 15_000 },
    workers: 1,
    reporter: [["line"]],
    use: {
        baseURL: "http://localhost:1420",
        // Pin the UI locale the same way the BDD config does: the specs
        // assert Russian product copy (e.g. the «Счётный суффикс» tag in
        // counter-scoring.spec.ts) and a fresh profile inherits the runner
        // locale otherwise.
        locale: "ru-RU",
        trace: "off",
        screenshot: "off",
        ...devices["Desktop Chrome"],
        // WASM + long onboarding flows: avoid tab crashes on /dev/shm
        launchOptions: {
            args: ["--disable-dev-shm-usage", "--disable-gpu"],
        },
    },
    globalSetup: "./global-setup.ts",
    globalTeardown: "./global-teardown.ts",
    webServer: [
        {
            command: "npx serve ../cdn -p 8080 --no-clipboard --cors",
            port: 8080,
            reuseExistingServer: !isCI,
            timeout: 30_000,
        },
        {
            // CI serves the prebuilt bundle (the same recipe as the BDD
            // config); locally `trunk serve` keeps the hot-reload loop.
            command: isCI
                ? "npx serve ../origa_ui/dist -s -p 1420 --no-clipboard"
                : "cd ../origa_ui && trunk serve",
            port: 1420,
            reuseExistingServer: !isCI,
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
