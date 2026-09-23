import { expect } from "@playwright/test";
import { When, Then, Given } from "../fixtures";
import { HomePage, OnboardingPage } from "../../pages";
import { getTrailBaseUrl } from "../../config";

// URL вынесен в константу: и локальный trunk-билд, и CI dist-билд
// компилируются с ORIGA_CDN_BASE_URL=http://localhost:8080 (end2end/.env
// и ci.yml e2e-build) — паттерн одинаков на обоих окружениях.
const CDN_URL = "http://localhost:8080/**";
const LOGIN_API = "**/api/auth/v1/login";

Given('CDN недоступен', async ({ page }) => {
    await page.context().route(CDN_URL, (route) => route.abort());
});

Then('каркас приложения отображается', async ({ page }) => {
    // Деградация без краша: карточка приложения отвечает, навигация
    // смонтирована (свежий юзер — пустое состояние главной).
    const homePage = new HomePage(page);
    await expect(homePage.sidebar).toBeVisible({ timeout: 30_000 });
});

// Бут при лежащем CDN блокируется экраном загрузки словарей (X из Y) —
// наблюдаемая деградация вместо чёрного экрана/краша.
Then('отображается экран загрузки словарей', async ({ page }) => {
    await expect(page.getByTestId("app-loading-overlay")).toBeVisible({
        timeout: 30_000,
    });
});

Given('сервер аутентификации недоступен', async ({ page }) => {
    await page.route(LOGIN_API, (route) => route.abort());
});

When('сервер аутентификации восстановился', async ({ page }) => {
    await page.unroute(LOGIN_API);
});

// Аборт всего TrailBase-трафика на живой странице онбординга бьёт ровно в
// save_sync скипа: get_current_user читает локальное хранилище (IndexedDB),
// мета наборов приходит с CDN — детерминированная целевая ветка.
//
// Локальный .env держит TRAILBASE_URL=http://127.0.0.1:4000, но dev-сборка
// WASM может резолвить дефолт http://localhost:4000 — хосты расходятся при
// одном порту. Абортим оба варианта, чтобы шаг не зависел от того, какая
// форма хоста досталась клиенту (в CI они совпадают — паттерны идентичны).
function trailbaseAbortPatterns(): string[] {
    const base = getTrailBaseUrl();
    const port = new URL(base).port;
    // Без явного порта в базовом URL localhost-вариант не строится — голый
    // `http://localhost:/**` никогда не совпадёт и только шумит.
    const patterns = [`${base}/**`];
    if (port) {
        patterns.push(`http://localhost:${port}/**`);
    }
    return [...new Set(patterns)];
}

Given('сервер приложения недоступен', async ({ page }) => {
    for (const pattern of trailbaseAbortPatterns()) {
        await page.context().route(pattern, (route) => route.abort());
    }
});

When('сервер приложения восстановился', async ({ page }) => {
    for (const pattern of trailbaseAbortPatterns()) {
        await page.context().unroute(pattern);
    }
});

When('пользователь скипает онбординг', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.skipOnboarding();
});

Then('отображается ошибка сохранения', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    // Тост появляется после исчерпания ретрая клиента (1 попытка +
    // 1.5s delay + попытка) — таймаут с запасом.
    await expect(onboarding.saveErrorToast).toBeVisible({ timeout: 15_000 });
});

Then('ошибка сохранения отображается в единственном экземпляре', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    // Синхронизация: повторный скип завершён ровно тогда, когда кнопка
    // снова включена (флаг сбрасывается тем же таском, что кладёт тост).
    // Без этого ожидания ассерт ловит СТАРЫЙ тост от первого фейла, пока
    // второй скип ещё в полёте. После разблокировки повторный фейл заменил
    // прежний тост по фиксированному id — ровно один элемент.
    await expect(onboarding.skipButton).toBeEnabled({ timeout: 15_000 });
    await expect(onboarding.saveErrorToast).toHaveCount(1);
});

Then('скип завершается переходом в приложение', async ({ page }) => {
    await page.waitForURL(/\/home/, { timeout: 30_000 });
});
