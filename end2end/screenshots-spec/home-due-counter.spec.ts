/**
 * Screenshots for issue #809: the due-card counter in the header of the
 * "Обзор дня" widget on the home page.
 *
 * The counter counts every card whose review date has come (domain
 * `MemoryHistory::is_due()`), regardless of the card's status — cards
 * marked "Уже знаю" included (user's decision, issue #809). Phrases and
 * examples are excluded: they have their own review pipelines.
 *
 * The seed makes the number deterministic: the onboarding "mark all known"
 * gives every corpus card a review date of YESTERDAY (see
 * `mark_card_as_known` in the domain), so an unpatched profile would show
 * the whole corpus as due. The seed therefore rewrites the dates: every
 * card gets a future date, and exactly DUE_TARGET cards get a past date.
 * `last_review_date` is bumped to "now" on every patched card — the
 * cross-device merge picks the later state, and a tie would hand the
 * server snapshot (yesterday) the win and break the expected number.
 *
 * Output directory: $SCREENSHOT_DIR (default: a temp dir — a plain run
 * must not leave binary screenshots in the working tree; the repo keeps
 * PR screenshots under .pr-screenshots/ and the factory uploads them to
 * the CDN).
 */
import { test, expect, type Page } from "@playwright/test";
import os from "node:os";
import path from "node:path";
import { LoginPage } from "../pages/login.page";
import { getAdminToken, createTestUser } from "../fixtures/admin";
import { completeOnboardingToScoring } from "../helpers/onboarding";
import { generateUniqueEmail, DEFAULT_TEST_PASSWORD } from "../helpers/auth";

const OUTPUT_DIR = path.resolve(
    process.env.SCREENSHOT_DIR ?? path.join(os.tmpdir(), "origa-pr-screenshots"),
);

/** How many cards the counter must show after the seed. */
const DUE_TARGET = 37;

/// RFC3339 without fractional seconds — the chrono-compatible domain format.
function rfc3339(date: Date): string {
    return date.toISOString().replace(/\.\d{3}Z$/, "Z");
}

function daysFromNow(days: number, hourUtc = 12): string {
    const date = new Date();
    date.setUTCDate(date.getUTCDate() + days);
    date.setUTCHours(hourUtc, 0, 0, 0);
    return rfc3339(date);
}

/// Phrases and examples are not counted by the counter (issue #809).
function isCountedType(card: Record<string, unknown>): boolean {
    const keys = Object.keys(card ?? {});
    return !keys.includes("Phrase") && !keys.includes("Example");
}

type StudyCard = {
    card?: Record<string, unknown>;
    memory_history?: {
        current_state?: { next_review_date?: string } | null;
        last_review_date?: string | null;
    };
};

type UserJson = {
    knowledge_set: {
        study_cards?: Record<string, StudyCard>;
    };
};

/// Cards whose review date has come — the number the counter shows.
function countDue(user: UserJson, now: Date): number {
    const cards = Object.values(user.knowledge_set?.study_cards ?? {});
    const nowMs = now.getTime();
    return cards.filter((card) => {
        const state = card.memory_history?.current_state;
        if (!state?.next_review_date) return false;
        if (!isCountedType(card.card ?? {})) return false;
        return new Date(state.next_review_date).getTime() <= nowMs;
    }).length;
}

/// Rewrites the review dates: every card gets a future date, exactly
/// DUE_TARGET countable cards get a past one. `last_review_date` is bumped
/// to "now" on every patched card so the local seed wins the LWW merge
/// against the server snapshot.
function seedDueCards(userJson: string, now: Date): { json: string; due: number } {
    const user = JSON.parse(userJson) as UserJson;
    const cards = Object.values(user.knowledge_set?.study_cards ?? {});

    const future = daysFromNow(30);
    const past = daysFromNow(-1);
    const nowStamp = rfc3339(now);

    let due = 0;
    for (const card of cards) {
        const memory = card.memory_history;
        // A card without a review state is new — the seed must not invent
        // one, that would move the card out of the "new" bucket.
        if (!memory?.current_state) continue;

        const countable = isCountedType(card.card ?? {});
        if (countable && due < DUE_TARGET) {
            memory.current_state.next_review_date = past;
            due += 1;
        } else {
            memory.current_state.next_review_date = future;
        }
        memory.last_review_date = nowStamp;
    }

    return { json: JSON.stringify(user), due };
}

async function readStoredUser(page: Page): Promise<{ key: string; value: string }> {
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
    return rawUser;
}

async function writeStoredUser(
    page: Page,
    key: string,
    value: string,
): Promise<void> {
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
        { key, value },
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
            // Russian UI: the login screen's language toggle decides the
            // native language of a brand-new profile.
            await page.getByTestId("lang-toggle-ru").click({ timeout: 10_000 });
            await login.fillEmail(email);
            await login.fillPassword(DEFAULT_TEST_PASSWORD);
            // The submit button stays disabled while the app loads its
            // dictionaries — clicking it early is a no-op.
            await expect(login.submitButton).toBeEnabled({ timeout: 180_000 });
            await login.submit();
            // The app lands on "/" (the home route), so the URL alone says
            // nothing — wait for the login screen to go away.
            await expect(login.loginPage).toBeHidden({ timeout: 120_000 });
            loggedIn = true;
        } catch {
            loggedIn = false;
        }
    }
    expect(loggedIn, "login must succeed within 3 attempts").toBe(true);

    // The app lands on "/" and shows the resource-loading overlay first.
    // The onboarding import needs those dictionaries: without them the
    // import runs against an empty dictionary and creates no cards. Once
    // the overlay is gone the home page mounts and redirects a fresh
    // profile to the onboarding.
    await expect(page.getByTestId("app-loading-overlay")).toBeHidden({
        timeout: 600_000,
    });
    await page.waitForURL(/\/onboarding$/, { timeout: 300_000 });

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
    await expect(page.getByTestId("home-page")).toBeVisible({
        timeout: 300_000,
    });
}

async function captureHome(
    page: Page,
    name: string,
    expectedCount?: number,
): Promise<void> {
    // The counter must be on screen before the shot — it is the feature
    // under review.
    const counter = page.getByTestId("home-today-overview-due-count");
    await expect(counter).toBeVisible({ timeout: 120_000 });
    if (expectedCount !== undefined) {
        // Exact number: a plain substring check would also pass for "137".
        await expect(counter).toHaveText(new RegExp(`\\b${expectedCount}\\b`), {
            timeout: 120_000,
        });
    }
    // Let fonts/images settle, then capture the viewport.
    await page.waitForTimeout(1_500);
    await page.screenshot({ path: path.join(OUTPUT_DIR, name) });
}

test("home due counter screenshots with seeded data", async ({ page }) => {
    test.setTimeout(900_000);

    // Desktop 1280x800
    await page.setViewportSize({ width: 1280, height: 800 });
    await loginFreshUser(page);

    // The realistic "day one" number: right after the onboarding marked
    // everything known, every corpus card has a review date of yesterday.
    const stored = await readStoredUser(page);
    const storedCards = Object.values(
        (JSON.parse(stored.value) as UserJson).knowledge_set?.study_cards ?? {},
    );
    console.log(
        `cards in the set: ${storedCards.length}, with a review state: ` +
            `${storedCards.filter((card) => card.memory_history?.current_state).length}`,
    );
    const dayOne = countDue(JSON.parse(stored.value) as UserJson, new Date());
    console.log(
        `day-one due cards (onboarding "mark all known", no date rewrite): ${dayOne}`,
    );

    // The day-one state is what a fresh profile really lands in — the
    // whole imported corpus is past its review date. Capture it before the
    // seed rewrites the dates.
    await captureHome(page, "home-due-counter-day-one-desktop.png", dayOne);

    const seeded = seedDueCards(stored.value, new Date());
    expect(seeded.due, "the seed must produce exactly DUE_TARGET due cards").toBe(
        DUE_TARGET,
    );
    await writeStoredUser(page, stored.key, seeded.json);

    // Reload re-reads the local record. The mount sync merges the server
    // snapshot with max()/LWW semantics — the seeded future dates survive
    // because every patched card carries a bumped `last_review_date`.
    await page.reload({ waitUntil: "domcontentloaded" });
    await captureHome(page, "home-due-counter-desktop.png", DUE_TARGET);

    // Mobile 390x844 — same seeded state.
    await page.setViewportSize({ width: 390, height: 844 });
    await page.reload({ waitUntil: "domcontentloaded" });
    await captureHome(page, "home-due-counter-mobile.png", DUE_TARGET);
});
