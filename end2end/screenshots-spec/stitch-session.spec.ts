/**
 * #749 final screenshots: sign the demo user in through the UI, inject the
 * seeded typical-user knowledge set directly into the app's IndexedDB
 * record, reload, and capture the RU insights ledger at both viewports.
 */
import { test, expect, type Page } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { LoginPage } from "../pages/login.page";
import { waitForLoginComplete } from "../helpers/auth";
import { getAdminToken, createTestUser } from "../fixtures/admin";

const EMAIL = "insights2@origa.local";
const PASSWORD = "e2e-test-password-123";
const PROPOSALS_DIR = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "../../.pr-screenshots/proposals-v2",
);

async function injectSeed(page: Page): Promise<void> {
    const seeded = await page.evaluate(async () => {
        const db = await new Promise<IDBDatabase>((resolve, reject) => {
            const request = indexedDB.open("origa");
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });
        const read = <T>(store: IDBObjectStore, request: IDBRequest<T>) =>
            new Promise<T>((resolve, reject) => {
                request.onsuccess = () => resolve(request.result);
                request.onerror = () => reject(request.error);
            });
        const tx = db.transaction("users", "readonly");
        const store = tx.objectStore("users");
        const keys = (await read(
            store,
            store.getAllKeys(),
        )) as IDBValidKey[];
        for (const key of keys) {
            if (typeof key === "string" && key.startsWith("user:")) {
                const value = (await read(
                    store,
                    store.get(key),
                )) as unknown;
                db.close();
                return { key, value: String(value) };
            }
        }
        db.close();
        throw new Error("no user record found in IndexedDB");
    });

    const user = JSON.parse(seeded.value);
    const ks = user.knowledge_set;
    const now = Date.now();
    const day = 86_400_000;
    const iso = (ms: number) => new Date(ms).toISOString().replace(/\.\d{3}Z$/, "Z");

    // История за обе недели темпа.
    const history = Array.isArray(ks.lesson_history) ? ks.lesson_history : [];
    for (const days of [2, 4, 6, 8, 10, 12, 15, 18, 21, 24, 27]) {
        const p = 8 + ((days * 3) % 4);
        const n = 2 + (days % 3);
        history.push({
            timestamp: iso(now - days * day),
            avg_stability: 12.4,
            avg_difficulty: 4.6,
            total_words: 24,
            new_words: 2,
            known_words: 10,
            in_progress_words: 10,
            high_difficulty_words: 2,
            new_cards_studied_today: 2,
            phrase_cards_studied_today: 1,
            lessons_completed: 1,
            positive_ratings: p,
            negative_ratings: n,
            total_ratings: p + n,
        });
    }
    ks.lesson_history = history;

    // Бакеты инсайтов: вечер худший, просрочка ~30 п.п.
    ks.insights = {
        hour_attempts: [2, 4, 8, 14, 30, 26],
        hour_correct: [2, 4, 7, 12, 22, 13],
        delay_attempts: [60, 20, 8, 4],
        delay_correct: [52, 13, 4, 1],
    };

    // Демо-слова: 24 штуки, у двух — провалы (для тяжёлых слов).
    const words: Array<[string, string]> = [
        ["結局", "in the end"], ["つまり", "in short"], ["それでも", "even so"],
        ["たとえば", "for example"], ["すなわち", "namely"], ["おそらく", "probably"],
        ["どうやって", "how"], ["やはり", "as expected"], ["もう一度", "once more"],
        ["できるだけ", "as much as possible"], ["やっぱり", "still"], ["けっこう", "considerably"],
        ["びっくり", "surprise"], ["ゆっくり", "slowly"], ["たぶん", "probably"],
        ["ほとんど", "mostly"], ["ちゃんと", "properly"], ["やっと", "at last"],
        ["きっと", "surely"], ["あまり", "not much"], ["とても", "very"],
        ["たくさん", "a lot"], ["少し", "a little"], ["みんな", "everyone"],
    ];
    words.forEach(([word, gloss], index) => {
        const ulid = `01J${String(index + 1).padStart(5, "0")}${"A".repeat(18)}`;
        const troubled = index < 2;
        const overdue = index % 2 === 0;
        const lastReview = now - (overdue ? 12 : 2) * day;
        ks.study_cards[ulid] = {
            card_id: ulid,
            card: {
                Vocabulary: {
                    word: { text: word },
                    reverse_side: { text: gloss },
                    pos: null,
                    tokens: null,
                },
            },
            memory_history: {
                current_state: {
                    stability: { value: troubled ? 4.2 : 14.8 },
                    difficulty: { value: troubled ? 7.4 : 4.6 },
                    next_review_date: iso(overdue ? lastReview + day : now + 5 * day),
                    card_state: "Review",
                },
                reps: troubled ? 9 : 6,
                lapses: troubled ? 4 : 0,
                easy_count: troubled ? 0 : 2,
                good_count: troubled ? 3 : 4,
                last_review_date: iso(lastReview),
                last_rating: troubled ? "Again" : "Good",
                consecutive_again: 0,
                ghost: null,
                marked_known_at: null,
            },
            is_favorite: false,
            perfect_streak_since_known: 0,
            favorite_changed_at: null,
        };
    });

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
        { key: seeded.key, value: JSON.stringify(user) },
    );
}

async function login(page: Page): Promise<void> {
    // The demo account is not part of the TrailBase fixture — create it so
    // the spec also runs on a fresh instance (CI). An existing account is
    // fine: the password is the same one the spec logs in with.
    const { token, csrfToken } = await getAdminToken();
    await createTestUser(token, csrfToken, EMAIL, PASSWORD).catch(() => undefined);

    await page.context().addInitScript(() => {
        if (window.location.origin === "http://localhost:1420") {
            window.localStorage.setItem("origa_resource_download_consented", "true");
            document.cookie = "i18n_pref_locale=ru; path=/; max-age=31536000";
        }
    });

    const login = new LoginPage(page);
    let loggedIn = false;
    for (let attempt = 1; attempt <= 5 && !loggedIn; attempt++) {
        await page.goto("/", { waitUntil: "domcontentloaded" });
        try {
            // Явные экшн-таймауты: клик по detached-элементу гоняется
            // с WASM-ре-рендером и вешает попытку целиком.
            const passwordToggle = page.getByTestId("login-password-toggle");
            await passwordToggle.waitFor({ state: "visible", timeout: 60_000 });
            await passwordToggle.click({ timeout: 10_000 });

            const emailInput = page.getByTestId("email-input");
            await emailInput.waitFor({ state: "visible", timeout: 30_000 });
            await emailInput.fill(EMAIL, { timeout: 10_000 });
            const passwordInput = page.getByTestId("password-input");
            await passwordInput.fill(PASSWORD, { timeout: 10_000 });
            const submit = page.getByTestId("login-submit");
            await submit.click({ timeout: 10_000 });

            await waitForLoginComplete(page);
            loggedIn = true;
        } catch {
            loggedIn = false;
        }
    }
    if (!loggedIn) {
        // UI-логин флакует на холодном WASM: часто сессия всё же установлена
        // (редирект случился после истечения waitForURL). Валидация ниже —
        // по фактическому наличию home-контента.
        console.warn("login loop ended without confirmed redirect; continuing");
    }
}

async function captureViewport(page: Page, name: string): Promise<void> {
    await expect(page.getByTestId("home-insights-week-verdict")).toBeVisible({
        timeout: 420_000,
    });
    await page.waitForTimeout(2_000);
    // Full page: леджер Наблюдений ниже счётчиков, обрезать нельзя.
    await page.screenshot({ path: path.join(PROPOSALS_DIR, name), fullPage: true });
}

test("capture home insights with seeded verdicts", async ({ page }) => {
    test.setTimeout(1_200_000);
    await login(page);

    // Приложение сохранило пользователя после логина — сидим его копию.
    await page.waitForTimeout(5_000);
    await injectSeed(page);

    // Desktop.
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.reload({ waitUntil: "domcontentloaded" });
    await captureViewport(page, "final-home-ru-desktop.png");

    // Mobile: hide the fixed tab bar — on a fullPage shot Playwright
    // stamps it mid-image, covering the ledger rows.
    await page.setViewportSize({ width: 390, height: 844 });
    await page.reload({ waitUntil: "domcontentloaded" });
    await captureViewport(page, "final-home-ru-mobile.png");
    // После reload: fixed таб-бар на fullPage-скрине штампуется в середину
    // изображения и перекрывает строки леджера.
    await page.addStyleTag({ content: ".bottom-tab-bar { display: none !important; }" });
    await page.waitForTimeout(300);
    await page.screenshot({ path: path.join(PROPOSALS_DIR, "final-home-ru-mobile.png"), fullPage: true });
});
