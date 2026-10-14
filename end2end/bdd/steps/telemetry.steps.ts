import { expect } from "@playwright/test";
import { Given, Then } from "../fixtures";
import {
    collectUmamiEvents,
    expectUmamiEventSequence,
    skipWhenUmamiTrackerAbsent,
    type UmamiEventCollector,
} from "../../helpers/analytics";

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

Then('трекер Umami отправил события всех шагов онбординга', async () => {
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
    await expectUmamiEventSequence(umamiCollector!, ONBOARDING_EVENT_FUNNEL);
});
