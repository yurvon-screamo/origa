/**
 * Screenshots for issue #749 (home insight panels): create a test user,
 * complete onboarding (mark all known → the corpus is in the set), then
 * seed a typical active-user state directly into the IndexedDB record:
 * three weeks of daily history, insight buckets with a worst evening
 * window and an overdue price, a couple of troubled words. Capture the
 * redesigned home with live verdicts — desktop and mobile. Output:
 * .pr-screenshots/ at the repo root.
 */
import { test, expect, type Page } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { LoginPage } from "../pages/login.page";
import { getAdminToken, createTestUser } from "../fixtures/admin";
import {
    completeOnboardingToScoring,
} from "../helpers/onboarding";
import { generateUniqueEmail, DEFAULT_TEST_PASSWORD } from "../helpers/auth";

const OUTPUT_DIR = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "../../.pr-screenshots",
);

/// RFC3339 без дробных секунд — chrono-совместимый формат домена.
function rfc3339(date: Date): string {
    return date.toISOString().replace(/\.\d{3}Z$/, "Z");
}

function daysAgoFromNow(days: number, hourUtc = 12): string {
    const date = new Date();
    date.setUTCDate(date.getUTCDate() - days);
    date.setUTCHours(hourUtc, 0, 0, 0);
    return rfc3339(date);
}

/**
 * Seeds a typical active-user state into the stored user JSON:
 * three weeks of daily ratings (both pace windows populated), insight
 * buckets whose worst window is the evening (>= MIN_SAMPLE), an overdue
 * price, and a couple of troubled words (lapses) for the top list.
 * All values follow the domain serialization the app itself writes.
 */
function seedTypicalUserState(userJson: string): string {
    const user = JSON.parse(userJson);
    const ks = user.knowledge_set;

    // 1. Daily history: ratings on ~13 days spread over both pace windows
    // (1..=14 and 15..=28 days back). Today is intentionally absent — the
    // featured verdict window includes today, but the pace windows must
    // stay full-day. NOTE: StatsTracker is #[serde(flatten)] inside
    // KnowledgeSet, so lesson_history sits at the TOP level of the
    // knowledge_set JSON — there is no "stats" key on the wire.
    const history: Array<Record<string, unknown>> = Array.isArray(
        ks.lesson_history,
    )
        ? ks.lesson_history
        : [];
    for (const days of [2, 4, 6, 8, 10, 12, 15, 18, 21, 24, 27]) {
        const dayPositive = 8 + ((days * 3) % 4);
        const dayNegative = 2 + (days % 3);
        history.push({
            timestamp: daysAgoFromNow(days),
            avg_stability: 12.4,
            avg_difficulty: 4.6,
            total_words: 320,
            new_words: 40,
            known_words: 190,
            in_progress_words: 78,
            high_difficulty_words: 12,
            new_cards_studied_today: 5,
            phrase_cards_studied_today: 2,
            lessons_completed: 2,
            positive_ratings: dayPositive,
            negative_ratings: dayNegative,
            total_ratings: dayPositive + dayNegative,
        });
    }
    ks.lesson_history = history;

    // 2. Insight buckets: evening (bucket 5) is the worst window with a
    // solid sample; overdue reviews lose ~34 p.p. against on-time.
    ks.insights = {
        hour_attempts: [2, 4, 8, 14, 30, 26],
        hour_correct: [2, 4, 7, 12, 22, 13],
        delay_attempts: [60, 20, 8, 4],
        delay_correct: [52, 13, 4, 1],
    };

    // 3. Troubled words: a realistic slice of the corpus (roughly one in
    // twenty-five) carries lapses so the share reads as "normal wear".
    const cards = ks.study_cards ?? {};
    const entries = Object.entries(cards);
    const troubledTarget = Math.max(3, Math.floor(entries.length / 25));
    let troubled = 0;
    for (const [, card] of entries) {
        const memory = (card as { memory_history?: Record<string, unknown> })
            .memory_history;
        if (!memory || !memory.current_state) continue;
        memory.lapses = troubled % 4 === 0 ? 4 : 3;
        memory.consecutive_again = 0;
        troubled += 1;
        if (troubled >= troubledTarget) break;
    }

    return JSON.stringify(user);
}

async function patchStoredUser(page: Page): Promise<void> {
    // The stored user is a JSON string in the `users` store (one record:
    // the fresh test context holds exactly one logged-in user).
    const rawUser = await page.evaluate(async () => {
        const db = await new Promise<IDBDatabase>((resolve, reject) => {
            const request = indexedDB.open("origa");
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });
        const tx = db.transaction("users", "readonly");
        const store = tx.objectStore("users");
        const keys = (await new Promise<IDBValidKey[]>((resolve, reject) => {
            const request = store.getAllKeys();
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        })) as IDBValidKey[];
        for (const key of keys) {
            if (typeof key !== "string" || !key.startsWith("user:")) continue;
            const value = (await new Promise<unknown>((resolve, reject) => {
                const request = store.get(key);
                request.onsuccess = () => resolve(request.result);
                request.onerror = () => reject(request.error);
            })) as unknown;
            if (typeof value === "string") {
                db.close();
                return { key, value };
            }
        }
        db.close();
        throw new Error("no JSON-string user record found in IndexedDB");
    });
    expect(rawUser.value.length, "user record must be non-empty").toBeGreaterThan(0);

    const seededJson = seedTypicalUserState(rawUser.value);

    await page.evaluate(
        ({ key, value }) =>
            new Promise<void>((resolve, reject) => {
                const open = indexedDB.open("origa");
                open.onerror = () => reject(open.error);
                open.onsuccess = () => {
                    const db = open.result;
                    const tx = db.transaction("users", "readwrite");
                    tx.objectStore("users").put(value, key);
                    tx.oncomplete = () => {
                        db.close();
                        resolve();
                    };
                    tx.onerror = () => reject(tx.error);
                };
            }),
        { key: rawUser.key, value: seededJson },
    );
}

async function loginFreshUser(page: Page): Promise<void> {
    const { token, csrfToken } = await getAdminToken();
    const email = generateUniqueEmail();
    await createTestUser(token, csrfToken, email, DEFAULT_TEST_PASSWORD);

    // Pre-approve the one-time resource download before any app script
    // runs — otherwise the consent overlay intercepts every click.
    await page.context().addInitScript(() => {
        if (window.location.origin === "http://localhost:1420") {
            window.localStorage.setItem("origa_resource_download_consented", "true");
        }
    });

    const login = new LoginPage(page);
    let loggedIn = false;
    for (let attempt = 1; attempt <= 3 && !loggedIn; attempt++) {
        await page.goto("/", { waitUntil: "domcontentloaded" });
        try {
            await login.expandPasswordForm();
            await login.fillEmail(email);
            await login.fillPassword(DEFAULT_TEST_PASSWORD);
            await login.submit();
            await page.waitForURL(/\/(home|onboarding)$/, { timeout: 120_000 });
            loggedIn = true;
        } catch {
            loggedIn = false;
        }
    }
    expect(loggedIn, "login must succeed within 3 attempts").toBe(true);

    if (page.url().includes("onboarding")) {
        // Full onboarding: the corpus lands in the knowledge set (cards
        // for the troubled-words top list), finish lands on /home.
        await completeOnboardingToScoring(page, { level: "N4" });
        const markAll = page.getByTestId("onboarding-mark-all-known");
        if (await markAll.isVisible({ timeout: 10_000 }).catch(() => false)) {
            await markAll.click();
            const confirmBtn = page.getByTestId("onboarding-confirm-ok");
            await confirmBtn.click({ timeout: 10_000 }).catch(() => undefined);
        }
        await expect(page.getByTestId("scoring-step-complete")).toBeVisible({
            timeout: 120_000,
        });
        const { OnboardingPage } = await import("../pages/onboarding.page");
        const onboarding = new OnboardingPage(page);
        await onboarding.clickFinish();
        await page.waitForURL(/\/home$/, { timeout: 300_000 });
    }
}

async function captureHome(page: Page, name: string): Promise<void> {
    await expect(
        page.getByTestId("home-insights-week-verdict"),
    ).toBeVisible({ timeout: 120_000 });
    // Let fonts/images settle, then capture the viewport.
    await page.waitForTimeout(1_500);
    await page.screenshot({ path: path.join(OUTPUT_DIR, name) });
}

test("home insights screenshots with seeded data", async ({ page }) => {
    test.setTimeout(900_000);

    // Desktop 1280x800
    await page.setViewportSize({ width: 1280, height: 800 });
    await loginFreshUser(page);
    await patchStoredUser(page);
    // Reload re-reads the local record. The mount sync merges the server
    // snapshot with max()/union semantics — the seeded counters and
    // history items survive the merge.
    await page.reload({ waitUntil: "domcontentloaded" });
    await captureHome(page, "home-insights-desktop.png");

    // Mobile 390x844 — same seeded state.
    await page.setViewportSize({ width: 390, height: 844 });
    await page.reload({ waitUntil: "domcontentloaded" });
    await captureHome(page, "home-insights-mobile.png");
});
