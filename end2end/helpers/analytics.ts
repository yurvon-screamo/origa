import { expect, test, type Page } from "@playwright/test";

/**
 * CI-build analytics guard (ADR-054).
 *
 * The dist served by e2e is compiled with `UMAMI_DISABLED=1`, so the Umami
 * tracker must not be present anywhere in the document. This guards the
 * build gate itself: if the CI env ever stops reaching the trunk build
 * (renamed variable, refactored step), CI data silently pollutes production
 * analytics — nothing else would fail.
 *
 * Skipped outside CI: local runs serve a regular trunk build where analytics
 * is enabled by default (mute locally by exporting `UMAMI_DISABLED=1` before
 * `npx playwright test` — the webServer inherits the environment).
 */
export async function expectUmamiTrackerAbsent(page: Page): Promise<void> {
    test.skip(!process.env.CI, "CI-build guard: dist outside CI keeps analytics enabled");
    await expect(page.locator("script[data-website-id]")).toHaveCount(0);
}

/**
 * Skips the calling scenario when the Umami tracker is compiled out
 * (`UMAMI_DISABLED=1` — the CI dist, or a local run following the README's
 * mute recommendation). Unlike {@link expectUmamiTrackerAbsent} this is a
 * runtime probe, not a build-gate assertion: telemetry-delivery scenarios
 * need the real tracker, so a muted build is a legitimate skip, not a
 * failure.
 */
export async function skipWhenUmamiTrackerAbsent(page: Page): Promise<void> {
    const trackerPresent = (await page.locator("script[data-website-id]").count()) > 0;
    test.skip(
        !trackerPresent,
        "трекер Umami скомпилирован out (UMAMI_DISABLED=1) — сценарий требует сборки с включённой аналитикой",
    );
}

/** One Umami event captured from a `POST /api/send` beacon. */
export interface UmamiCapturedEvent {
    name: string;
    /** Event data property, when the tracker attached one. */
    locale?: string;
    device?: string;
}

/** Read handle for events captured by {@link collectUmamiEvents}. */
export interface UmamiEventCollector {
    events: UmamiCapturedEvent[];
}

/**
 * Installs a page-level pass-through interception of the Umami collection
 * endpoint that records event payloads. Install BEFORE the walk starts
 * (routing applies to requests issued after the call); the tracker sends
 * via `sendBeacon`, so the body arrives with a text/plain content-type and
 * must be parsed from `postData()`.
 *
 * Callers poll {@link UmamiEventCollector.events} — beacons land
 * asynchronously after the triggering action.
 */
export async function collectUmamiEvents(page: Page): Promise<UmamiEventCollector> {
    const collector: UmamiEventCollector = { events: [] };
    await page.route("**/api/send", async (route) => {
        const request = route.request();
        if (request.method() === "POST") {
            const body = request.postData();
            if (body) {
                try {
                    const parsed = JSON.parse(body) as {
                        payload?: { name?: string; data?: Record<string, unknown> };
                    };
                    if (parsed.payload?.name) {
                        collector.events.push({
                            name: parsed.payload.name,
                            locale: parsed.payload.data?.locale as string | undefined,
                            device: parsed.payload.data?.device as string | undefined,
                        });
                    }
                } catch {
                    // Not the payload shape we track — pass it through.
                }
            }
        }
        await route.continue();
    });
    return collector;
}

/**
 * Asserts `expected` appears in the collector as an ordered subsequence
 * (pageviews and unrelated events may interleave), checking the declared
 * event-data properties on each matched instance.
 */
export async function expectUmamiEventSequence(
    collector: UmamiEventCollector,
    expected: Array<{ name: string; locale?: string; device?: string }>,
): Promise<void> {
    const names = collector.events.map((event) => event.name);
    let cursor = 0;
    for (const expectedEvent of expected) {
        const offset = names.slice(cursor).indexOf(expectedEvent.name);
        expect(
            offset,
            `событие "${expectedEvent.name}" не найдено после позиции ${cursor} в [${names.join(", ")}]`,
        ).toBeGreaterThanOrEqual(0);
        cursor += offset + 1;
        const matched = collector.events[cursor - 1];
        if (expectedEvent.locale !== undefined) {
            expect(matched.locale, `locale у "${expectedEvent.name}"`).toBe(expectedEvent.locale);
        }
        if (expectedEvent.device !== undefined) {
            expect(matched.device, `device у "${expectedEvent.name}"`).toBe(expectedEvent.device);
        }
    }
}
