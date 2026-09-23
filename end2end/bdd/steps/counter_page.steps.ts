import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";

const COUNTERS_URL = "http://localhost:1420/counters";

When("пользователь открывает страницу счётчиков", async ({ page }) => {
	await page.goto(COUNTERS_URL);
	await page.getByTestId("counters-card").waitFor({ timeout: 30_000 });
});

Then("страница счётчиков показывает пустое состояние", async ({ page }) => {
	await expect(page.getByTestId("counters-empty-state")).toBeVisible({
		timeout: 15_000,
	});
});

When(
	"пользователь открывает дровер добавления суффиксов",
	async ({ page }) => {
		await page.getByTestId("counters-add-btn").click();
		await page.getByTestId("counters-add-drawer").waitFor({ timeout: 15_000 });
		await page.getByTestId("counters-drawer-item").first().waitFor({
			state: "visible",
			timeout: 30_000,
		});
	},
);

Then("в дровере доступны тайлы суффиксов", async ({ page }) => {
	const tiles = page.getByTestId("counters-drawer-item");
	const count = await tiles.count();
	expect(count, "N5 registry tiles must render").toBeGreaterThanOrEqual(10);
});

When(
	"пользователь выбирает тайл суффикса {string}",
	async ({ page }, suffix: string) => {
		await page
			.getByTestId("counters-drawer-item", { hasText: suffix })
			.first()
			.click();
	},
);

When("пользователь подтверждает добавление из дровера", async ({ page }) => {
	await page.getByTestId("counters-drawer-add-btn").click();
	await page.getByTestId("counters-add-drawer").waitFor({
		state: "hidden",
		timeout: 30_000,
	});
});

Then(
	"в списке счётчиков появляется карточка суффикса {string}",
	async ({ page }, suffix: string) => {
		const card = page.getByTestId("counter-card-item", { hasText: suffix });
		await expect(card).toBeVisible({ timeout: 15_000 });
		await expect(card.getByTestId("counter-card-bindings")).toBeVisible();
	},
);

When(
	"пользователь открывает детальную карточку суффикса {string}",
	async ({ page }, suffix: string) => {
		await page
			.getByTestId("counter-card-item", { hasText: suffix })
			.first()
			.click();
		await page.getByTestId("counters-detail-hero").waitFor({
			timeout: 15_000,
		});
	},
);

Then(
	"в детальной карточке видна таблица чтений суффикса",
	async ({ page }) => {
		const table = page.getByTestId("counters-detail-table");
		await expect(table).toBeVisible({ timeout: 10_000 });
		const rows = table.locator(".counter-readings-row");
		// 1..=10 + 何 — фиксированный инвариант датасета.
		expect(await rows.count()).toBeGreaterThanOrEqual(11);
	},
);

Then("breadcrumbs ведут назад к списку счётчиков", async ({ page }) => {
	const link = page.locator(".counter-breadcrumbs a");
	await expect(link).toBeVisible();
	await expect(link).toHaveAttribute("href", "/counters");
});

When(
	"пользователь помечает суффикс известным из деталей",
	async ({ page }) => {
		await page.getByTestId("counters-detail-actions-mark-known-btn").click();
	},
);

Then(
	"все связки суффикса изучены в детальной карточке",
	async ({ page }) => {
		const summary = page.getByTestId("counters-detail-bindings");
		await expect(summary).toBeVisible({ timeout: 20_000 });
		await expect
			.poll(async () => summary.textContent(), { timeout: 30_000 })
			.toContain("11/11");
	},
);

When(
	"пользователь добавляет суффикс в избранное из карточки",
	async ({ page }) => {
		await page
			.getByTestId("counter-card-item-favorite-btn")
			.first()
			.click();
	},
);

Then("карточка суффикса в избранном", async ({ page }) => {
	// Сердце меняет только SVG-fill — пользовательски значимый признак
	// избранного: карточка видна под статус-фильтром «Избранные».
	await page.getByTestId("counters-filter-favorite").click();
	await expect(
		page.getByTestId("counter-card-item").first(),
	).toBeVisible({ timeout: 15_000 });
});

When("пользователь удаляет суффикс из карточки", async ({ page }) => {
	await page.getByTestId("counter-card-item-delete-btn").first().click();
	await page.getByTestId("counter-delete-modal").waitFor({
		state: "visible",
		timeout: 10_000,
	});
	await page.getByTestId("counter-delete-modal-confirm").click();
});

Then("страница счётчиков снова пуста", async ({ page }) => {
	await expect(page.getByTestId("counters-empty-state")).toBeVisible({
		timeout: 20_000,
	});
});
