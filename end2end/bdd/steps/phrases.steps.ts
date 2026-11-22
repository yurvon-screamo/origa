import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { PhrasesPage, WordsPage } from "../../pages";
import { waitUntilStable } from "../../helpers/wait";

/**
 * The injected study card's id (a syntactically valid ULID) — the same
 * card the `phrase-dash-run` shots preset seeds.
 */
const DASH_RUN_CARD_ID = "PM4Q528M101GNGSPPPZCY2PHFH";

Given(
  "в профиле есть просроченная карточка фразы {string}",
  async ({ page }, phraseId: string) => {
    // Persisted-schema injection, the counter seed step's contract:
    // db=origa, store=users, nested JSON string — a schema drift there
    // needs a synchronous edit here. The words flow above has created
    // the user row; the full /home navigation below re-reads the store
    // with the patched card.
    await page.evaluate(
      ({ cardId, phraseId }) =>
        new Promise<void>((resolve, reject) => {
          const open = indexedDB.open("origa");
          open.onerror = () => reject(open.error);
          open.onsuccess = () => {
            const db = open.result;
            const tx = db.transaction("users", "readwrite");
            const cur = tx.objectStore("users").openCursor();
            cur.onsuccess = () => {
              const c = cur.result;
              if (!c) return;
              const v = c.value;
              if (typeof v === "string" && v.includes('"email"')) {
                try {
                  const u = JSON.parse(v);
                  u.knowledge_set = u.knowledge_set ?? {};
                  u.knowledge_set.study_cards =
                    u.knowledge_set.study_cards ?? {};
                  u.knowledge_set.study_cards[cardId] = {
                    card_id: cardId,
                    card: { Phrase: { phrase_id: phraseId } },
                    memory_history: {
                      current_state: {
                        stability: { value: 3.0 },
                        difficulty: { value: 5.0 },
                        next_review_date: "2020-01-01T00:00:00Z",
                        card_state: "Review",
                      },
                      reps: 1,
                      lapses: 0,
                      easy_count: 0,
                      good_count: 1,
                      last_review_date: "2020-01-01T00:00:00Z",
                      last_rating: "Good",
                      consecutive_again: 0,
                    },
                  };
                  c.update(JSON.stringify(u));
                } catch {
                  /* skip non-JSON rows */
                }
              }
              c.continue();
            };
            tx.oncomplete = () => {
              db.close();
              resolve();
            };
            tx.onerror = () => reject(tx.error);
          };
        }),
      { cardId: DASH_RUN_CARD_ID, phraseId },
    );
    await page.goto("/home");
    await page.waitForURL(/\/home$/, { timeout: 120_000 });
  },
);

When("пользователь открывает страницу фраз", async ({ page }) => {
  const phrasesPage = new PhrasesPage(page);
  await phrasesPage.goto();
  await phrasesPage.expectPhrasesVisible();
});

When(
  "ищет фразу {string} на странице фраз",
  async ({ page }, query: string) => {
    const phrasesPage = new PhrasesPage(page);
    await phrasesPage.searchInput.fill(query);
  },
);

When("нажимает навигацию к фразам", async ({ page }) => {
  // #823: phrases left the nav bars — the entry point is the words
  // header button now.
  const wordsPage = new WordsPage(page);
  await wordsPage.phrasesButton.click({ timeout: 10_000 });
  await page.waitForURL(/\/phrases$/, { timeout: 10_000 });
});

Then("страница фраз отображается", async ({ page }) => {
  const phrasesPage = new PhrasesPage(page);
  await expect(phrasesPage.phrasesPage).toBeVisible({ timeout: 15_000 });
});

Then("поле поиска фраз содержит {string}", async ({ page }, query: string) => {
  const phrasesPage = new PhrasesPage(page);
  await expect(phrasesPage.searchInput).toHaveValue(query, { timeout: 10_000 });
});

Then("на странице фраз отображается пустое состояние", async ({ page }) => {
  const phrasesPage = new PhrasesPage(page);
  await expect(phrasesPage.emptyState).toBeVisible();
});

Then("отображается поле поиска фраз", async ({ page }) => {
  const phrasesPage = new PhrasesPage(page);
  await expect(phrasesPage.searchInput).toBeVisible();
});

When("нажимает кнопку возврата с фраз", async ({ page }) => {
  await page.goto("/home");
  await page.waitForURL(/\/home$/, { timeout: 10_000 });
});

Then("отображаются кнопки фильтрации фраз", async ({ page }) => {
  await expect(
    page.getByTestId(/phrase.*filter|filter.*phrase/).first(),
  ).toBeVisible({ timeout: 10_000 });
});

Then(
  "карточка фразы {string} отображается без тире-ранов",
  async ({ page }, phraseText: string) => {
    const item = page.getByTestId("phrases-card-item").first();
    await expect(item).toBeVisible({ timeout: 60_000 });

    // The phrase text renders through TranslatorText: the one-off
    // dictionary warmup shows the loader first (#521) — wait it out.
    await expect(item.getByTestId("translator-dict-loading")).toHaveCount(0, {
      timeout: 60_000,
    });

    // Token surfaces concatenate in the DOM, so the clean phrase reads
    // as one string; the dirty prod bytes must never reach the card.
    const body = (await item.textContent()) ?? "";
    expect(body).toContain(phraseText);
    expect(
      /[\u{2500}\u{2015}\u{2014}]{2}/u.test(body),
      `dash-run artifact leaked onto the rendered card: ${body}`,
    ).toBe(false);
  },
);

Then("карточки фраз имеют непустой текст", async ({ page }) => {
  await expect(page.getByTestId("phrases-card-item").first()).toBeVisible({
    timeout: 30_000,
  });
  const text = await page
    .getByTestId("phrases-card-item")
    .first()
    .textContent();
  expect(text?.trim().length ?? 0).toBeGreaterThan(0);
});

// Main CJK ideograph block only — a deliberate subset of the
// translator's has_kanji ranges (Ext-A/compat omitted): any card this
// filter picks is guaranteed to grow a ruby, which is the direction
// the assertion needs. No need to sync this regex with the translator
// gate if the gate ever widens.
const KANJI_RE = /[\u4E00-\u9FFF]/;

Then("текст фраз содержит фуригану", async ({ page }) => {
  // Furigana arrives either from the precompute blob (#521 fast path)
  // or from the background tokenizer warmup — generous timeout covers
  // the latter in CI (~344 MB dictionary download from the mirror).
  // Anchored to the card (not the translator span): the translator's
  // no-language fallback branch renders plain text without inner
  // test ids, and the card is the stable container either way.
  //
  // The card is picked by kanji presence, NOT by list position (#584):
  // the first card may legitimately be an all-kana phrase — the
  // translator gates ruby on has_kanji (translator.rs), so it renders
  // no ruby by design — and the fresh-user card order floats (Ulid
  // sort), which made ruby-in-first-card a data gamble that flaked
  // this scenario. A kanji-bearing card must always grow a ruby once
  // its tokens render; that invariant is the furigana contract this
  // step pins.
  const kanjiCard = page
    .getByTestId("phrases-card-item")
    .filter({ hasText: KANJI_RE })
    .first();
  await expect(kanjiCard).toBeVisible({ timeout: 30_000 });
  await expect(
    kanjiCard.locator(".furigana-ruby").first(),
    "kanji-bearing phrase card must render furigana ruby",
  ).toBeVisible({ timeout: 120_000 });
});

When("ищет фразы {string}", async ({ page }, query: string) => {
  const phrasesPage = new PhrasesPage(page);
  // Шаг-действие: заполнили поиск. Дальше поиск догружает детали фраз
  // в фоне, и следующий Then сам ждёт результат (toHaveCount с
  // авто-ретраем) — фиксированная пауза здесь только задерживала
  // прогон и ничего не гарантировала.
  await phrasesPage.searchPhrases(query);
});

Then("на странице фраз нет карточек", async ({ page }) => {
  await expect(page.getByTestId("phrases-card-item")).toHaveCount(0, {
    timeout: 10_000,
  });
});

/**
 * Network log across the phrases-page-load When-step of the lazy-load
 * scenario (#540-В1): the Then-step counts phrase data chunk requests.
 */
let phrasesRequestLog: string[] = [];

When(
  "пользователь открывает страницу фраз с записью сетевых запросов",
  async ({ page }) => {
    phrasesRequestLog = [];
    page.on("request", (request) => phrasesRequestLog.push(request.url()));
    const phrasesPage = new PhrasesPage(page);
    await phrasesPage.goto();
    await phrasesPage.expectPhrasesVisible();
  },
);

Then(
  "количество загруженных чанков данных фраз ограничено видимой пачкой",
  async () => {
    const chunkUrl = /\/phrases\/data\/p\d{4}\.json/;
    const countNow = () =>
      phrasesRequestLog.filter((url) => chunkUrl.test(url)).length;

    // Wait for the request stream to stabilize: two consecutive samples
    // with the same count mean the burst is over. The pre-#540 behavior
    // (fetch EVERY chunk of the user's ~6k phrases) kept the stream busy
    // for a long time and would sail far past the bound below; the lazy
    // visible-slice load settles quickly on a few dozen chunks.
    let settleError = "";
    await waitUntilStable(countNow, {
      label: "phrase data chunk requests",
      quietMs: 1_000,
      timeoutMs: 10_000,
      pollMs: 500,
    }).catch((error: unknown) => {
      // Поток не устоялся — уходим к проверке ниже: она сообщает
      // фактическое число загруженных чанков, это и есть actionable
      // сигнал (как и раньше, когда цикл просто исчерпывал итерации).
      settleError = error instanceof Error ? error.message : String(error);
    });

    const finalCount = countNow();
    expect(
      finalCount,
      `the lazy phrases page must load only the visible slice's chunks (#540), got ${finalCount}` +
        (settleError
          ? ` — the request stream never settled: ${settleError}`
          : ""),
    ).toBeLessThanOrEqual(60);
    // Positive control: data WAS loaded (the preceding step already
    // asserted non-empty card text, this guards against a vacuous zero).
    expect(
      finalCount,
      "the visible slice must actually fetch its chunks",
    ).toBeGreaterThan(0);
  },
);

When("удаляет первую фразу", async ({ page }) => {
  await page
    .getByTestId("phrases-card-item")
    .first()
    .locator('[data-testid*="delete"]')
    .first()
    .click();
});

Then("отображается сообщение о подтверждении удаления", async ({ page }) => {
  const phrasesPage = new PhrasesPage(page);
  await expect(phrasesPage.deleteModal).toBeVisible();
});
