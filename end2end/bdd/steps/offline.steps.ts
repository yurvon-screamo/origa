import { expect, type Page } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { getTrailBaseUrl } from "../../config";

/**
 * Offline startup scenarios (ADR-052): the app must open from cache or
 * show an actionable error — never spin forever on the loading overlay.
 *
 * "Интернет недоступен" is emulated by aborting every request EXCEPT the
 * app origin: the page itself (localhost:1420) keeps loading, while the
 * CDN and TrailBase are dead. That models the real-world "connected
 * Wi-Fi, no uplink" grey zone — and, unlike context.setOffline, works
 * across reloads.
 */

const CDN_URL = "http://localhost:8080/**";
const APP_ORIGIN = "http://localhost:1420";

/** Cache API stores owned by the dictionary layer (cdn_provider.rs,
 * dictionary_cache.rs). localStorage/IndexedDB hold the session and the
 * user record — those must survive a dictionary-cache wipe. */
const ORIGA_CACHE_PREFIX = "origa-";

/** Every vocabulary fallback layer: the CDN rkyv blob, the 11 JSON chunks
 * (both keyed in origa-cdn-v1) and the parsed rkyv cache store. A partial
 * wipe leaves a live layer and silently green-tests the happy path. */
const VOCABULARY_CACHE_HINTS = [
    "/dictionary/vocabulary.rkyv",
    "/dictionary/chunk_",
    "/__origa_vocabulary_cached__",
];

async function abortExternalTraffic(page: Page): Promise<void> {
    const trailBaseUrl = getTrailBaseUrl();
    await page.context().route(CDN_URL, (route) => route.abort());
    await page.context().route(`${trailBaseUrl}/**`, (route) => route.abort());
}

async function restoreExternalTraffic(page: Page): Promise<void> {
    const trailBaseUrl = getTrailBaseUrl();
    await page.context().unroute(CDN_URL);
    await page.context().unroute(`${trailBaseUrl}/**`);
}

/** Wait until the authenticated shell finished its first dictionary boot:
 * the terminal navigation happened and the loading overlay is gone. */
async function waitForAppBoot(page: Page): Promise<void> {
    await page
        .waitForURL(/\/(home|onboarding|words|sets|profile|grammar|kanji|phrases|lesson)/, {
            timeout: 90_000,
        })
        .catch(() => {
            console.warn(`[offline] terminal navigation timeout; url=${page.url()}`);
        });
    await page
        .getByTestId("app-loading-overlay")
        .waitFor({ state: "detached", timeout: 180_000 });
}

function isAppUrl(page: Page): boolean {
    return page.url().startsWith(APP_ORIGIN);
}

// ── Given ────────────────────────────────────────────────────────────

Given('пользователь ранее вошёл в аккаунт', async ({ page }) => {
    await waitForAppBoot(page);
});

Given('пользователь ранее загрузил словари', async ({ page }) => {
    await waitForAppBoot(page);
});

Given('сессия аутентификации сохранена', async ({ page }) => {
    await waitForAppBoot(page);
});

Given('кэш словарей пуст', async ({ page }) => {
    await page.evaluate(async () => {
        const keys = await caches.keys();
        for (const key of keys) {
            if (key.startsWith("origa-")) await caches.delete(key);
        }
    });
});

Given('кэш словаря слов очищен полностью', async ({ page }) => {
    await page.evaluate(async () => {
        const keys = await caches.keys();
        for (const key of keys) {
            if (key === "origa-vocabulary-rkyv-v1") {
                await caches.delete(key);
                continue;
            }
            if (key !== "origa-cdn-v1") continue;
            const cache = await caches.open(key);
            const requests = await cache.keys();
            for (const request of requests) {
                const url = request.url;
                if (
                    url.includes("/dictionary/vocabulary.rkyv") ||
                    url.includes("/dictionary/chunk_")
                ) {
                    await cache.delete(request);
                }
            }
        }
    });
});

Given('локальный профиль пользователя отсутствует', async ({ page }) => {
    // Surgical wipe: clear only the `users` object store of the app DB.
    // The TrailBase session lives in localStorage and must survive — the
    // scenario asserts the merge-branch behaviour of check_session.
    await page.evaluate(
        () =>
            new Promise<void>((resolve, reject) => {
                const request = indexedDB.open("origa");
                request.onsuccess = () => {
                    const db = request.result;
                    const tx = db.transaction("users", "readwrite");
                    tx.objectStore("users").clear();
                    tx.oncomplete = () => {
                        db.close();
                        resolve();
                    };
                    tx.onerror = () => reject(tx.error);
                };
                request.onerror = () => reject(request.error);
            }),
    );
});

Given('интернет недоступен', async ({ page }) => {
    await abortExternalTraffic(page);
});

// ── When ─────────────────────────────────────────────────────────────

When('пользователь открывает приложение', async ({ page, offlineNetLog }) => {
    page.on("request", (request) => {
        if (request.url().startsWith("http://localhost:8080")) {
            offlineNetLog.requests.push(request.url());
        }
    });
    page.on("response", (response) => {
        if (response.url().startsWith("http://localhost:8080") && response.status() < 400) {
            offlineNetLog.okResponses.push(response.url());
        }
    });

    if (isAppUrl(page)) {
        await page.reload({ waitUntil: "domcontentloaded" });
    } else {
        await page.goto(APP_ORIGIN, { waitUntil: "domcontentloaded" });
    }
});

When('интернет становится доступен', async ({ page }) => {
    await restoreExternalTraffic(page);
});

When('пользователь нажимает кнопку «Повторить»', async ({ page }) => {
    await page.getByTestId("app-load-error-retry").click({ timeout: 10_000 });
});

// ── Then ─────────────────────────────────────────────────────────────

Then('отображается экран ошибки загрузки', async ({ page }) => {
    await expect(page.getByTestId("app-load-error")).toBeVisible({ timeout: 30_000 });
});

Then('не позже чем через 20 секунд отображается экран ошибки загрузки', async ({ page }) => {
    await expect(page.getByTestId("app-load-error")).toBeVisible({ timeout: 20_000 });
});

Then('отображается кнопка «Повторить»', async ({ page }) => {
    await expect(page.getByTestId("app-load-error-retry")).toBeVisible({ timeout: 5_000 });
});

Then('экран ошибки загрузки не отображается', async ({ page }) => {
    await expect(page.getByTestId("app-load-error")).toBeHidden({ timeout: 5_000 });
});

Then('экран загрузки словарей не отображается', async ({ page }) => {
    await expect(page.getByTestId("app-loading-overlay")).toBeHidden({ timeout: 5_000 });
});

Then('не позже чем через 20 секунд вечный экран загрузки не отображается', async ({ page }) => {
    await expect(page.getByTestId("app-loading-overlay")).toBeHidden({ timeout: 20_000 });
});

Then('вход в приложение завершается', async ({ page }) => {
    await page.waitForURL(/\/(home|onboarding)/, { timeout: 180_000 });
    await page
        .getByTestId("app-loading-overlay")
        .waitFor({ state: "detached", timeout: 180_000 });
});

Then('не позже чем через 10 секунд приложение открывается', async ({ page }) => {
    await expect(page.getByTestId("app-loading-overlay")).toBeHidden({ timeout: 10_000 });
    await expect(
        page.getByTestId("sidebar").or(page.getByTestId("onboarding-skip")),
    ).toBeVisible({ timeout: 10_000 });
});

Then('не позже чем через 20 секунд приложение открывается', async ({ page }) => {
    await expect(page.getByTestId("app-loading-overlay")).toBeHidden({ timeout: 20_000 });
    await expect(
        page.getByTestId("sidebar").or(page.getByTestId("onboarding-skip")),
    ).toBeVisible({ timeout: 10_000 });
});

Then('кнопка входа снова доступна', async ({ page }) => {
    await expect(page.getByTestId("login-submit")).toBeEnabled({ timeout: 10_000 });
});

Then('от CDN нет успешных ответов', async ({ offlineNetLog }) => {
    expect(
        offlineNetLog.okResponses,
        `CDN must not answer successfully offline, got: ${offlineNetLog.okResponses.join(", ")}`,
    ).toHaveLength(0);
});

Then('допускается только один безуспешный запрос манифеста', async ({ offlineNetLog }) => {
    const manifestRequests = offlineNetLog.requests.filter((url) =>
        url.includes("manifest.json"),
    );
    expect(
        manifestRequests.length,
        `at most one manifest probe is allowed, got: ${manifestRequests.join(", ")}`,
    ).toBeLessThanOrEqual(1);
});

Then('аудио фраз урока доступно из кэша', async ({ page }) => {
    // Cards without recorded audio are legitimate; but any audio element
    // that IS present must play from a cached blob: URL, never a raw CDN
    // URL (the gzip-on-CDN decoding contract, cdn_provider.rs).
    const audioElements = page.locator("audio");
    const count = await audioElements.count();
    for (let i = 0; i < count; i++) {
        const src = await audioElements.nth(i).getAttribute("src");
        expect(src, "lesson audio must use a cached blob: URL").toMatch(/^blob:/);
    }
});
