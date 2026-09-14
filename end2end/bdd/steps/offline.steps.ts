import { expect, type Page } from "@playwright/test";
import { Before, Given, When, Then, test } from "../fixtures";
import { getTrailBaseUrl } from "../../config";

/**
 * Offline startup scenarios (ADR-053): the app must open from cache or
 * show an actionable error — never spin forever on the loading overlay.
 *
 * "Интернет недоступен" is emulated by aborting every request EXCEPT the
 * app origin: the page itself (localhost:1420) keeps loading, while the
 * CDN and TrailBase are dead. That models the real-world "connected
 * Wi-Fi, no uplink" grey zone — and, unlike context.setOffline, works
 * across reloads.
 */

// The offline lesson scenario seeds a word (tokenizer download +
// inflation) and then walks the full lesson — the default 180 s test
// budget does not cover the seeding on CI runners.
Before('@slow-startup', () => {
    test.info().setTimeout(420_000);
});

const CDN_URL = "http://localhost:8080/**";
const APP_ORIGIN = "http://localhost:1420";

/** Cache API stores owned by the dictionary layer (cdn_provider.rs,
 * dictionary_cache.rs). localStorage/IndexedDB hold the session and the
 * user record — those must survive a dictionary-cache wipe. */
const ORIGA_CACHE_PREFIX = "origa-";

/** Every vocabulary fallback layer as URL fragments: the CDN rkyv blob
 * and the 11 JSON chunks (both keyed in origa-cdn-v1). A partial wipe
 * leaves a live layer and silently green-tests the happy path. The
 * parsed rkyv store is deleted whole (its name is a store, not a URL). */
const VOCABULARY_CACHE_HINTS = [
    "/dictionary/vocabulary.rkyv",
    "/dictionary/chunk_",
];
const VOCABULARY_RKYV_STORE = "origa-vocabulary-rkyv-v1";

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
    await page.evaluate(
        ([hints, rkyvStore]) => {
            return (async () => {
                const keys = await caches.keys();
                for (const key of keys) {
                    if (key === rkyvStore) {
                        await caches.delete(key);
                        continue;
                    }
                    if (key !== "origa-cdn-v1") continue;
                    const cache = await caches.open(key);
                    const requests = await cache.keys();
                    for (const request of requests) {
                        if (hints.some((hint) => request.url.includes(hint))) {
                            await cache.delete(request);
                        }
                    }
                }
            })();
        },
        [VOCABULARY_CACHE_HINTS, VOCABULARY_RKYV_STORE] as const,
    );
});

Given('локальный профиль пользователя отсутствует', async ({ page }) => {
    // Surgical wipe: clear only the `users` object store of the app DB
    // (names mirror DB_NAME/STORE_NAME in file_repository.rs — keep them
    // in sync). The TrailBase session lives in localStorage and must
    // survive — the scenario asserts the merge-branch behaviour of
    // check_session.
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
    // The scenario asserts the error screen BEFORE unrouting the
    // network: restoring connectivity mid-pipeline lets the manifest
    // and dictionaries download for real, and the screen never appears.
    await page.getByTestId("app-load-error-retry").click({ timeout: 10_000 });
});

Then('видна страница входа', async ({ page }) => {
    // 60 s covers the cold CI WASM boot; the shared login-page helper
    // only waits 15 s for the password toggle.
    await expect(page.getByTestId("login-password-toggle")).toBeVisible({
        timeout: 60_000,
    });
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
