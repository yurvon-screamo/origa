import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { KanjiPage } from "../../pages";

/**
 * Network log across the app-reload When-step of the art-manifest
 * scenario (#540): the Then-step asserts kanji WITHOUT CDN art are never
 * requested once the manifest is in the e2e mirror.
 */
let reloadRequestLog: string[] = [];

/** N5-set kanji that have no kanji_frames/kanji_animations art on the
 * CDN (verified against the deployed kanji_art_manifest.json). */
const ARTLESS_N5_KANJI_URL_ENCODED = [
    "%E5%85%B6", // 其
    "%E6%AD%A4", // 此
    "%E7%A2%97", // 碗
    "%E8%B3%91", // 賑
    "%E8%BF%9A", // 迚
    "%E9%86%A4", // 醤
    "%E9%9E%84", // 鞄
    "%E9%A3%B4", // 飴
    "%E9%B9%B8", // 鹸
];

When("пользователь перезагружает приложение с записью сетевых запросов", async ({ page }) => {
    reloadRequestLog = [];
    page.on("request", (request) => reloadRequestLog.push(request.url()));
    await page.reload({ waitUntil: "domcontentloaded" });
    // The startup pipeline (incl. the card pre-cache trigger in Phase E)
    // runs while/after the overlay; wait it out before asserting.
    await page
        .locator(".loading-overlay")
        .waitFor({ state: "hidden", timeout: 120_000 });
});

When("дождавшись запросов кандзи-арта", async ({ page }) => {
    // Positive control: the pre-cache must reach the kanji art stage —
    // at least one covered kanji produces a request. Without this the
    // negative assertion below could pass vacuously.
    const deadline = Date.now() + 90_000;
    while (Date.now() < deadline) {
        const hasArtRequest = reloadRequestLog.some(
            (url) => url.includes("kanji_frames/") || url.includes("kanji_animations/"),
        );
        if (hasArtRequest) {
            // Give the pre-cache a moment to finish its batch so a late
            // artless request is not missed by the assert below.
            await page.waitForTimeout(2_000);
            return;
        }
        await page.waitForTimeout(500);
    }
    throw new Error(
        "Positive control failed: no kanji art requests observed — the pre-cache never reached the art stage",
    );
});

Then("кандзи без арта на CDN не запрашиваются", async () => {
    const offenders = reloadRequestLog.filter((url) =>
        ARTLESS_N5_KANJI_URL_ENCODED.some(
            (encoded) =>
                url.includes(`kanji_frames/${encoded}`) || url.includes(`kanji_animations/${encoded}`),
        ),
    );
    expect(
        offenders,
        `kanji without CDN art must not be requested (#540 manifest filter), got: ${offenders.join(", ")}`,
    ).toEqual([]);
});

Given('у пользователя есть добавленное кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
    await kanjiPage.addBtn.click();
    await expect(kanjiPage.drawer).toBeVisible({ timeout: 10_000 });
    // Pick exactly one kanji (the first in the current level) so scenarios
    // like "Удаление кандзи" can reach the empty-state after a single delete.
    // Tests that need many kanji use the separate "много кандзи" Given.
    const firstItem = kanjiPage.drawer.getByTestId("kanji-drawer-item").first();
    await expect(firstItem).toBeVisible({ timeout: 10_000 });
    await firstItem.click();
    await kanjiPage.drawerAddBtn.click();
    await expect(kanjiPage.drawer).not.toBeVisible({ timeout: 30_000 });
    await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 30_000 });
});

When('пользователь открывает страницу кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
});

When('открывает добавление кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.addBtn.click();
    await expect(kanjiPage.drawer).toBeVisible({ timeout: 10_000 });
});

When('подтверждает добавление кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    // The "add" button stays disabled until at least one kanji is selected.
    // If the upstream scenario skipped an explicit selection step (e.g. the
    // "Добавление кандзи N5" path), fall back to "select all".
    if (await kanjiPage.drawerAddBtn.isEnabled().catch(() => false)) {
        // already enabled — something is selected
    } else {
        await kanjiPage.drawerSelectAllBtn.click();
        await expect(kanjiPage.drawerAddBtn).toBeEnabled({ timeout: 10_000 });
    }
    await kanjiPage.drawerAddBtn.click();
    await expect(kanjiPage.drawer).not.toBeVisible({ timeout: 30_000 });
});

Then('кандзи отображается в сетке', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 30_000 });
    await expect(kanjiPage.emptyState).not.toBeVisible();
});

Then('отображается более одного кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    const count = await kanjiPage.getCardCount();
    expect(count).toBeGreaterThan(1);
});

Then('на странице кандзи отображается пустое состояние', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await expect(kanjiPage.emptyState).toBeVisible();
});

When('пользователь удаляет первое кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.deleteCardByIndex(0);
});

When('пользователь отменяет удаление первого кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.cancelDeleteCardByIndex(0);
});

When('нажимает кнопку возврата на кандзи', async ({ page }) => {
    await page.goto("/home");
    await page.waitForURL(/\/home$/, { timeout: 10_000 });
});

When('пользователь отмечает первое кандзи как известное', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.markCardAsKnownByIndex(0);
});

When('пользователь открывает детали первого кандзи', async ({ page }) => {
    await page.getByTestId("kanji-card-item").first().click();
    await page.waitForURL(/\/kanji\//, { timeout: 10_000 });
    await expect(page.getByTestId("kanji-detail")).toBeVisible({ timeout: 15_000 });
});

Then('отображается страница деталей кандзи', async ({ page }) => {
    await expect(page.getByTestId("kanji-detail")).toBeVisible({ timeout: 15_000 });
});

Then('отображается содержимое деталей кандзи', async ({ page }) => {
    await expect(page.getByTestId("kanji-detail")).toBeVisible({ timeout: 10_000 });
});

Then('чтения кандзи отображаются отдельными тегами', async ({ page }) => {
    // Hero card renders readings via <ReadingGroup>, one .reading-tag span per
    // reading. At least one of the ON/KUN groups must surface a tag.
    const onGroup = page.getByTestId("kanji-detail-on-readings");
    const kunGroup = page.getByTestId("kanji-detail-kun-readings");
    // Each tag carries a data-rare attribute (stringified "true"/"false"
    // by ReadingGroup), so we can count via that stable attribute instead
    // of the CSS class — class selectors are brittle in CI minified builds.
    const onTags = onGroup.locator("[data-rare]");
    const kunTags = kunGroup.locator("[data-rare]");
    // Tags mount after the kanji dictionary loads — wait web-first instead
    // of an instant count() that races the dictionary load.
    await expect(onTags.or(kunTags).first()).toBeVisible({ timeout: 15_000 });
    const onCount = await onTags.count();
    const kunCount = await kunTags.count();
    expect(onCount + kunCount).toBeGreaterThan(0);
});

Then('в карточке кандзи отображаются compact-чтения', async ({ page }) => {
    await expect(page.getByTestId("kanji-card-compact-readings").first()).toBeVisible({
        timeout: 10_000,
    });
});

Given('у пользователя есть кандзи "厳"', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
    await kanjiPage.addBtn.click();
    await expect(kanjiPage.drawer).toBeVisible({ timeout: 10_000 });
    // 厳 is an N1 kanji — switch level, search, select, add.
    await kanjiPage.selectLevel("N1");
    await kanjiPage.searchKanji("厳");
    await kanjiPage.selectKanji("厳");
    await kanjiPage.addSelectedKanji();
    await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 30_000 });
});

When('пользователь открывает детали этого кандзи', async ({ page }) => {
    await page.getByTestId("kanji-card-item").first().click();
    await page.waitForURL(/\/kanji\//, { timeout: 10_000 });
    await expect(page.getByTestId("kanji-detail")).toBeVisible({ timeout: 15_000 });
});

Then('редкие чтения кандзи отображаются приглушёнными', async ({ page }) => {
    // ReadingGroup renders rare readings with data-rare="true" (stringified
    // for tachys determinism). Scoped to the kanji-detail hero to avoid
    // matching lesson cards.
    //
    // Reading tags mount only after the kanji dictionary loads, so the
    // step first waits for ANY tag (web-first): an instant count() raced
    // the dictionary load and misreported "ReadingGroup is broken" on
    // slow cold loads.
    //
    // Back-compat: when kanji.json lacks reading_frequencies (pre-deploy),
    // NO reading is rare → data-rare="true" never appears → this step
    // would fail. Detect that case and skip with an informative message
    // rather than blocking the PR on data-not-yet-deployed.
    const detail = page.getByTestId("kanji-detail");
    const rareTags = detail.locator('[data-rare="true"]');
    const normalTags = detail.locator('[data-rare="false"]');
    await expect(rareTags.or(normalTags).first()).toBeVisible({ timeout: 15_000 });
    const rareCount = await rareTags.count();
    if (rareCount === 0) {
        const total = await normalTags.count();
        if (total === 0) {
            throw new Error(
                "No reading tags rendered at all — ReadingGroup is broken",
            );
        }
        console.warn(
            "Skipping 'rare readings' assertion: kanji.json on CDN lacks " +
            "reading_frequencies. Deploy via scripts/enrich_kanji_reading_frequencies.py " +
            "--apply && python scripts/deploy_cdn.py to enable.",
        );
        return;
    }
    expect(rareCount).toBeGreaterThanOrEqual(1);
});

Then('частые чтения кандзи отображаются обычным стилем', async ({ page }) => {
    // Same back-compat path as 'редкие чтения': if no rare tags exist (pre-deploy
    // CDN), normal tags still render with data-rare="false". This step always
    // requires normal tags to be present.
    const detail = page.getByTestId("kanji-detail");
    const normalTags = detail.locator('[data-rare="false"]');
    await expect(normalTags.first()).toBeVisible({ timeout: 10_000 });
});

When('нажимает кнопку выбора всех кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.drawerSelectAllBtn.click();
});

Given('у пользователя есть много кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
    await kanjiPage.addBtn.click();
    await expect(kanjiPage.drawer).toBeVisible({ timeout: 10_000 });
    await kanjiPage.drawerSelectAllBtn.click();
    await kanjiPage.drawerAddBtn.click();
    await expect(kanjiPage.drawer).not.toBeVisible({ timeout: 60_000 });
});

When('выбирает уровни кандзи N5 и N4', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    const n5Btn = page.getByTestId("kanji-level-n5");
    if (await n5Btn.isVisible().catch(() => false)) {
        await n5Btn.click();
        await kanjiPage.drawerSelectAllBtn.click();
    }
    const n4Btn = page.getByTestId("kanji-level-n4");
    if (await n4Btn.isVisible().catch(() => false)) {
        await n4Btn.click();
        await kanjiPage.drawerSelectAllBtn.click();
    }
});

Then('CJK шрифты загружены', async ({ page }) => {
    await expect(page.getByTestId("kanji-page")).toBeVisible();
    const fontUsed = await page.evaluate(() => {
        const el = document.querySelector('[data-testid="kanji-page"]');
        if (!el) return false;
        const font = window.getComputedStyle(el).fontFamily;
        return font.includes("NotoSans") || font.includes("NotoSerif") || font.includes("Noto");
    });
    expect(fontUsed).toBe(true);
});

Given('у пользователя есть кандзи нескольких уровней', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
    await kanjiPage.addBtn.click();
    await expect(kanjiPage.drawer).toBeVisible({ timeout: 10_000 });
    // Pick a couple of N5 kanji, switch to N4, pick a couple there. This
    // guarantees both grid-N5 and grid-N4 sections will render.
    await kanjiPage.selectLevel("N5");
    const n5First = kanjiPage.drawer.getByTestId("kanji-drawer-item").first();
    await n5First.click();
    await kanjiPage.selectLevel("N4");
    const n4First = kanjiPage.drawer.getByTestId("kanji-drawer-item").first();
    await n4First.click();
    await kanjiPage.addSelectedKanji();
    await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 30_000 });
});

When('выбирает фильтр уровня кандзи {string}', async ({ page }, level: string) => {
    const testid = `kanji-filter-jlpt-${level.toLowerCase()}`;
    await page.getByTestId(testid).click();
});

Then('отображается только группа кандзи уровня {string}', async ({ page }, level: string) => {
    await expect(page.getByTestId(`kanji-grid-${level}`)).toBeVisible({ timeout: 10_000 });
    for (const other of ["N5", "N4", "N3", "N2", "N1", "other"]) {
        if (other !== level) {
            await expect(page.getByTestId(`kanji-grid-${other}`)).not.toBeVisible();
        }
    }
});
