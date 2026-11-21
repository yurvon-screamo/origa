import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { HomePage, KanjiPage } from "../../pages";
import {
  apiLogin,
  DEFAULT_TEST_PASSWORD,
  generateUniqueEmail,
  wipeClientAuthState,
} from "../../helpers/auth";
import { createTestUser } from "../../fixtures/admin";
import { getTrailBaseUrl } from "../../config";
import { fetchWithTimeout } from "../../helpers/http";

When("пользователь переходит на страницу слов", async ({ page }) => {
  await page.goto("/words");
  await page.waitForLoadState("networkidle");
});

When("пользователь перезагружает страницу", async ({ page }) => {
  await page.reload();
  await page.waitForLoadState("networkidle");
});

When("возвращается на главную страницу", async ({ page }) => {
  const homePage = new HomePage(page);
  await homePage.goto();
});

When("нажимает навигацию к кандзи", async ({ page }) => {
  const homePage = new HomePage(page);
  await homePage.sidebarKanji.click();
});

Then("страница кандзи отображается", async ({ page }) => {
  const kanjiPage = new KanjiPage(page);
  await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
});

// --- Ремедиация #1449: гвард бут-минта против сессии с пустым email ---

// Хвост реального OAuth-края: колбэк персистит серверную сессию ДО проверки
// email (email_not_in_token) — в хранилище остаётся сессия с настоящим
// токеном и пустым email. Свежая учётка здесь — как настоящий OAuth-юзер:
// auth-запись есть, профиль нигде не создан (ни локально, ни на сервере).
// Cleanup учётки — глобальный orphan-sweeper (префикс e2e-%).
Given(
  "в хранилище лежит валидная сессия без email",
  async ({ page, testUser }) => {
    const freshEmail = generateUniqueEmail();
    await createTestUser(
      testUser.adminToken,
      testUser.adminCsrfToken,
      freshEmail,
      DEFAULT_TEST_PASSWORD,
    );
    await wipeClientAuthState(page);
    // apiLogin сеет реальную одноразовую запись (бросовый документ на
    // статическом ассете, WASM не бутается) — тот же шов, что у фикстуры.
    await apiLogin(page.context(), freshEmail, DEFAULT_TEST_PASSWORD);
    // Пустой email поверх валидного сида: токен настоящий (все серверные
    // запросы аутентифицированы), поле email — "". Точная симуляция хвоста
    // OAuth-колбэка.
    await page.evaluate(() => {
      const raw = window.localStorage.getItem("trailbase_session");
      if (!raw) throw new Error("session seed missing after apiLogin");
      const session = JSON.parse(raw) as { email: string };
      session.email = "";
      window.localStorage.setItem("trailbase_session", JSON.stringify(session));
    });
  },
);

Then("профиль с пустым email не создан на сервере", async ({ testUser }) => {
  // Каноническая запись минтится в domain_user (клиент v12+), легаси-путь —
  // user; гвард обязан удержать обе. Пустой email у канонической записи —
  // однозначный след минта из сессии без email (живые юзеры приходят с
  // настоящими адресами).
  const response = await fetchWithTimeout(
    `${getTrailBaseUrl()}/api/_admin/query`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${testUser.adminToken}`,
        "csrf-token": testUser.adminCsrfToken,
      },
      body: JSON.stringify({
        query:
          "SELECT (SELECT COUNT(*) FROM user WHERE email = '') + (SELECT COUNT(*) FROM domain_user WHERE email = '') as cnt",
      }),
    },
    10_000,
  );
  expect(response.ok, "admin query for empty-email profiles").toBeTruthy();
  const data = (await response.json()) as { rows: [[{ Integer: number }]] };
  const emptyEmailProfiles = data.rows?.[0]?.[0]?.Integer ?? -1;
  expect(
    emptyEmailProfiles,
    "boot mint from an empty-email session pushed a canonical record",
  ).toBe(0);
});

Then("отображается боковая панель навигации", async ({ page }) => {
  const homePage = new HomePage(page);
  await expect(homePage.sidebar).toBeVisible();
});
