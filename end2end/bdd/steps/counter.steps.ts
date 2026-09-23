import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { HomePage } from "../../pages";
import { awaitHandVisible } from "../../helpers/lesson";

When("пользователь начинает урок со счётными суффиксами", async ({ page }) => {
	// Диагностика day-1 (issue #415): слушатель ставится ДО перехода,
	// чтобы поймать панику WASM на домашней странице — сценарии падают
	// молчаливым таймаутом на home-welcome-lesson.
	const diagnostics = attachPageDiagnostics(page, "start-lesson");
	try {
		const homePage = new HomePage(page);
		await homePage.goto();
		await homePage.startLesson();
	} catch (err) {
		console.info(`[counter-diagnostics:start-lesson-failure]\n${diagnostics()}`);
		throw err;
	}
});

/// Временная диагностика (issue #417, day-1 flake): собирает консоль и
/// pageerror текущей страницы и прикладывает к отчёту — упавший сценарий
/// покажет WASM-панику вместо молчаливого таймаута.
export function attachPageDiagnostics(page: import("@playwright/test").Page, label: string) {
	const logs: string[] = [];
	page.on("console", (msg) => logs.push(`[${msg.type()}] ${msg.text()}`));
	page.on("pageerror", (err) => logs.push(`[pageerror] ${err.message}`));
	return () => {
		const text = logs.join("\n") || "(no console output)";
		console.info(`[counter-diagnostics:${label}]\n${text}`);
		return text;
	};
}

/// Показ руки: жмём «Дальше», пока не встретим слайд счётного суффикса
/// (issue #415: миграция заводит counter-карту из слова «一本»; в маленьком
/// пуле рука содержит и слово, и счётчик — слайд гарантированно в показе).
Then(
	"в показе руки знакомства появляется слайд счётного суффикса",
	async ({ page }) => {
		const diagnostics = attachPageDiagnostics(page, "presentation");
		try {
			await awaitHandVisible(page);
		const nextBtn = page.getByTestId("acquaintance-next-btn");
		const counterSlide = page.getByTestId("acquaintance-counter-slide");
		for (let i = 0; i < 20; i++) {
			if (await counterSlide.isVisible().catch(() => false)) return;
			await nextBtn.click({ timeout: 3_000 }).catch(() => null);
			// Показ мог закончиться тренировкой — слайд был пропущен.
			const training = await page
				.getByTestId("acquaintance-training")
				.waitFor({ state: "visible", timeout: 500 })
				.then(() => true)
				.catch(() => false);
			if (training) break;
		}
		await expect(counterSlide).toBeVisible({ timeout: 2_000 });
		} catch (err) {
			console.info(`[counter-diagnostics:presentation-failure]\n${diagnostics()}`);
			throw err;
		}
	},
);

Then(
	"слайд счётного суффикса несёт таблицу чтений",
	async ({ page }) => {
		await expect(
			page.getByTestId("acquaintance-counter-slide-table"),
		).toBeVisible();
		// 本-фикстура датасета: 1..=10 + 何 — таблица длинная, скроллится.
		const rows = page.getByTestId("acquaintance-counter-slide-table").locator("> div");
		expect(await rows.count()).toBeGreaterThanOrEqual(11);
	},
);

Then(
	"в тренировке руки счётный суффикс спрашивается знаком",
	async ({ page }) => {
		const training = page.getByTestId("acquaintance-training");
		await training.waitFor({ state: "visible", timeout: 15_000 });
		// Ищем фронт-знак счётного суффикса среди показанных фронтов.
		// Рука может перемешивать карты — пролистываем виток ответами
		// «Не помню» (сброс не ломает инвариант: фронт счётчика — знак).
		const front = page.getByTestId("acquaintance-training-front");
		let found = false;
		for (let i = 0; i < 14 && !found; i++) {
			found = await front
				.getByText("本", { exact: true })
				.first()
				.isVisible()
				.catch(() => false);
			if (found) break;
			const answer = page.getByTestId("acquaintance-reveal-btn");
			await answer.click({ timeout: 2_000 }).catch(() => null);
			const forgot = page.getByTestId("acquaintance-rating-dont-know");
			await forgot.click({ timeout: 2_000 }).catch(() => null);
		}
		expect(found, "счётный суффикс не встретился в тренировке").toBe(true);
	},
);
