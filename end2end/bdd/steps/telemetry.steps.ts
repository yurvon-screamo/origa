import { expect } from "@playwright/test";
import { Given, Then } from "../fixtures";
import {
    collectUmamiEvents,
    expectUmamiEventSequence,
    skipWhenUmamiTrackerAbsent,
    type UmamiEventCollector,
} from "../../helpers/analytics";
import { completeOnboardingToScoring } from "../../helpers/onboarding";

// Collector for the telemetry scenario below. Scenario-scoped: playwright-bdd
// runs scenarios of a file sequentially within a worker (same pattern as the
// relay-user list in onboarding_scoring.steps.ts), so a single slot is safe.
let umamiCollector: UmamiEventCollector | null = null;

// The full step-wise funnel of issue #774: every onboarding step start and
// completion in walk order, ending with the declarative onboarding_complete
// of the scoring finish button. locale/device come from the event data
// properties (e2e pins locale ru-RU; Desktop Chrome classifies as desktop).
const ONBOARDING_EVENT_FUNNEL = [
    { name: "onboarding_intro_start", locale: "ru", device: "desktop" },
    { name: "onboarding_intro_complete", locale: "ru", device: "desktop" },
    { name: "onboarding_load_start", locale: "ru", device: "desktop" },
    { name: "onboarding_load_complete", locale: "ru", device: "desktop" },
    { name: "onboarding_jlpt_start", locale: "ru", device: "desktop" },
    { name: "onboarding_jlpt_complete", locale: "ru", device: "desktop" },
    { name: "onboarding_apps_start", locale: "ru", device: "desktop" },
    { name: "onboarding_apps_complete", locale: "ru", device: "desktop" },
    { name: "onboarding_progress_start", locale: "ru", device: "desktop" },
    { name: "onboarding_progress_complete", locale: "ru", device: "desktop" },
    { name: "onboarding_summary_start", locale: "ru", device: "desktop" },
    { name: "onboarding_summary_complete", locale: "ru", device: "desktop" },
    { name: "onboarding_scoring_start", locale: "ru", device: "desktop" },
    { name: "onboarding_complete", locale: "ru", device: "desktop" },
];

Given('перехват запросов аналитики Umami', async ({ page }) => {
    // Installed before any onboarding walk: only requests issued after this
    // call are captured, so nothing from the fixture login pollutes the log.
    umamiCollector = await collectUmamiEvents(page);
});

Given('новый пользователь с включённой аналитикой', async ({ page }) => {
    await page.waitForURL(/\/(home|onboarding)/, { timeout: 30_000 });
    await skipWhenUmamiTrackerAbsent(page);
});

// Telemetry-specific walk: identical to the shared empty-import Given, plus
// a daily-load click — an intra-step state mutation proving that start
// events fire on step ENTRY, not on every interaction (issue #774 review).
Given('новый пользователь дошёл до шага сводки импорта с кликом темпа загрузки', async ({ page }) => {
    const reached = await completeOnboardingToScoring(page, {
        level: "none",
        skipApps: true,
        stopAtSummary: true,
        selectDailyLoad: true,
    });
    expect(reached, "onboarding must stop at the summary step").toBe(true);
});

Then('трекер Umami отправил события всех шагов онбординга', async ({ page }) => {
    expect(umamiCollector, "перехват должен быть установлен раньше хода").not.toBeNull();
    // Beacons land asynchronously after the finish click — wait for the last
    // funnel event before asserting the ordered sequence.
    await expect
        .poll(
            () =>
                umamiCollector?.events.some((event) => event.name === "onboarding_complete") ??
                false,
            { timeout: 15_000 },
        )
        .toBe(true);

    // Exact equality: the collector holds precisely the funnel, no duplicate
    // starts (intra-step mutations must not re-fire them) and no extras.
    // Navigation entries are in the message for failure forensics: the
    // sessionStorage dedup survives reloads, so a duplicate start here
    // means a second wasm session (fresh tab / storage failure), not a
    // reload mid-walk.
    const navigations = await page.evaluate(() =>
        performance
            .getEntriesByType("navigation")
            .map(
                (entry) =>
                    `${(entry as PerformanceNavigationTiming).type}:${new URL(entry.name).pathname}`,
            )
            .join(","),
    );

    const names = umamiCollector!.events.map((event) => event.name);
    const expectedNames = ONBOARDING_EVENT_FUNNEL.map((event) => event.name);
    expect(
        names,
        `точная воронка: без дублей start и лишних событий; navigations=[${navigations}]`,
    ).toEqual(expectedNames);

    await expectUmamiEventSequence(umamiCollector!, ONBOARDING_EVENT_FUNNEL);
});
