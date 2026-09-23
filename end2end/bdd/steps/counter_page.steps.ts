import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";

const COUNTERS_URL = "http://localhost:1420/counters";

/// Суффикс-карточка в сетке: ссылка /counters/:suffix с тестидом
/// counters-item-suffix внутри.
function gridItem(page: import("@playwright/test").Page, suffix: string) {
	return page
		.locator("[data-testid=counters-grid] a", {
			has: page.locator(`[data-testid=counters-item-suffix]`, {
				hasText: suffix,
			}),
		})
		.first();
}

When("пользователь открывает страницу счётчиков", async ({ page }) => {
	await page.goto(COUNTERS_URL);
	await page.getByTestId("counters-grid").waitFor({ timeout: 30_000 });
});

Then("на странице счётчиков видна сетка суффиксов", async ({ page }) => {
	const items = page.locator("[data-testid=counters-item-suffix]");
	const count = await items.count();
	expect(count, "registry of 76 counters must render").toBeGreaterThanOrEqual(70);
});

When("пользователь открывает детальную карточку суффикса {string}", async ({ page }, suffix: string) => {
	await gridItem(page, suffix).click();
	await page.getByTestId("counters-detail").waitFor({ timeout: 15_000 });
	await page.getByTestId("counters-detail-head").waitFor({ timeout: 15_000 });
});

Then("в детальной карточке видна таблица чтений суффикса", async ({ page }) => {
	const table = page.getByTestId("counters-detail-table");
	await expect(table).toBeVisible({ timeout: 10_000 });
	const rows = table.locator("> div");
	const count = await rows.count();
	// 1..=10 + 何 — фиксированный инвариант датасета для любого суффикса.
	expect(count).toBeGreaterThanOrEqual(11);
	// 何 — последняя строка таблицы.
	const last = await rows.last().textContent();
	expect(last, "何 comes last").toContain("何");
});

When("пользователь открывает модалку добавления суффиксов", async ({ page }) => {
	await page.getByTestId("counters-add-open").click();
	await page.getByTestId("counters-add-modal").waitFor({ timeout: 10_000 });
	await page.getByTestId("counters-add-list").waitFor({ timeout: 10_000 });
});

When("пользователь выбирает суффикс {string} в модалке", async ({ page }, suffix: string) => {
	const item = page
		.locator("[data-testid=counters-add-list] button", { hasText: suffix })
		.first();
	await item.click();
});

When("пользователь подтверждает добавление суффиксов", async ({ page }) => {
	await page.getByTestId("counters-add-confirm").click();
	// Модалка закрывается по завершении операции.
	await page.getByTestId("counters-add-modal").waitFor({ state: "hidden", timeout: 20_000 });
});

Then("суффикс {string} отмечен в колоде на странице счётчиков", async ({ page }, suffix: string) => {
	const item = gridItem(page, suffix);
	await expect(item).toBeVisible();
	await expect(item.getByTestId("counters-item-in-deck")).toBeVisible({ timeout: 10_000 });
	await expect(item.getByTestId("counters-item-bindings")).toBeVisible();
});

When("пользователь добавляет суффикс из детальной карточки", async ({ page }) => {
	await page.getByTestId("counters-detail-add").click();
	await page.getByTestId("counters-detail-actions").waitFor({ timeout: 20_000 });
});

When("пользователь помечает суффикс известным", async ({ page }) => {
	await page.getByTestId("counters-detail-known").click();
	// Все связки получают известную память: сводка доезжает N/N.
	const summary = page.getByTestId("counters-detail-bindings");
	await expect(summary).toBeVisible({ timeout: 20_000 });
	await expect.poll(async () => summary.textContent(), { timeout: 30_000 }).toContain("/11");
});

Then("все связки суффикса изучены в детальной карточке", async ({ page }) => {
	// «Уже знаю» сидирует каждую связку: сводка показывает полный охват,
	// строки таблицы подсвечены как изученные.
	const summary = page.getByTestId("counters-detail-bindings");
	await expect(summary).toBeVisible();
	const text = (await summary.textContent()) ?? "";
	expect(text, `bindings summary must be full, got: ${text}`).toMatch(/11\s*\/\s*11/);
});

When("пользователь добавляет суффикс в избранное", async ({ page }) => {
	await page.getByTestId("counters-detail-favorite").click();
});

Then("суффикс в избранном на детальной карточке", async ({ page }) => {
	// Кнопка переключается в состояние «убрать из избранного».
	await expect(page.getByTestId("counters-detail-favorite")).toBeVisible();
	await expect
		.poll(
			async () => page.getByTestId("counters-detail-favorite").textContent(),
			{ timeout: 20_000 },
		)
		.not.toContain("избранное");
});

When("пользователь удаляет суффикс", async ({ page }) => {
	await page.getByTestId("counters-detail-delete").click();
});

Then("суффикс снова предлагается добавить в детальной карточке", async ({ page }) => {
	await page.getByTestId("counters-detail-add").waitFor({ timeout: 20_000 });
	// Карточка больше не в колоде: блок действий исчез.
	await expect(page.getByTestId("counters-detail-actions")).toHaveCount(0);
});
