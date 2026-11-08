import assert from "node:assert/strict";
import { mock, test } from "node:test";

import type { Browser, BrowserContext, Page } from "@playwright/test";

import type { RunOptions } from "./runner.ts";
import type { ShotEntry } from "./manifest.ts";
import type { ShotContext, ShotSink } from "./types.ts";
import type { ViewportTag } from "./viewports.ts";
import { DEFAULT_VIEWPORTS } from "./viewports.ts";

/**
 * The viewport plan of a capture, pinned by unit tests.
 *
 * `runScenario` decides WHICH viewports a scenario walks. Its boundaries
 * (browser session, sink, CDN probe, the scenario implementation) are
 * replaced with module mocks — the plan itself is the behaviour under
 * test: every scenario walks the desktop+mobile pair unless the run
 * narrows it with an explicit `viewports` override (the CLI `--viewports`
 * flag). The mocks must be registered before the runner is imported —
 * ESM hoists static imports, so the runner comes in dynamically below.
 */

// Viewports the fake scenario walked, in run order.
const walkedViewports: ViewportTag[] = [];

mock.module("./session.ts", {
    namedExports: {
        openShotSession: async (): Promise<{
            context: BrowserContext;
            page: Page;
            close(): Promise<void>;
        }> => ({ context: {} as BrowserContext, page: {} as Page, close: async () => {} }),
    },
});

mock.module("./capture.ts", {
    namedExports: {
        createSink: (_page: Page, options: { viewport: ViewportTag }): ShotSink => {
            const entries: ShotEntry[] = [];
            return {
                entries,
                shot: async (name: string) => {
                    walkedViewports.push(options.viewport);
                    const entry = { screen: name, viewport: options.viewport } as ShotEntry;
                    entries.push(entry);
                    return entry;
                },
            };
        },
    },
});

mock.module("./bundle.ts", {
    namedExports: {
        detectCdnBaseUrl: async () => "http://cdn.test:8080",
    },
});

mock.module("./scenarios/index.ts", {
    namedExports: {
        getImplementation: () => ({
            id: "runner-mock",
            title: "Runner mock",
            route: "/home",
            locale: "ru-RU" as const,
            preset: "fresh" as const,
            run: async (_context: ShotContext, sink: ShotSink) => {
                await sink.shot("screen");
            },
        }),
    },
});

const { runScenario } = await import("./runner.ts");

const browser = {} as Browser;

function runOptions(overrides: Partial<RunOptions> = {}): RunOptions {
    return {
        outDir: "/tmp/opencode/shots-runner-test",
        cdnBaseUrl: "http://cdn.test:8080",
        commit: "test",
        ...overrides,
    };
}

test("a scenario walks the desktop+mobile pair by default", async () => {
    walkedViewports.length = 0;
    const entries = await runScenario(browser, "runner-mock", runOptions());

    assert.deepEqual(walkedViewports, ["desktop", "mobile"]);
    assert.deepEqual(
        entries.map((entry: ShotEntry) => entry.viewport),
        ["desktop", "mobile"],
    );
});

test("an explicit viewports override narrows the walk", async () => {
    walkedViewports.length = 0;
    const entries = await runScenario(browser, "runner-mock", runOptions({ viewports: ["mobile"] }));

    assert.deepEqual(walkedViewports, ["mobile"]);
    assert.deepEqual(
        entries.map((entry: ShotEntry) => entry.viewport),
        ["mobile"],
    );
});

test("the default pair is exactly desktop and mobile", () => {
    assert.deepEqual(DEFAULT_VIEWPORTS, ["desktop", "mobile"]);
});
