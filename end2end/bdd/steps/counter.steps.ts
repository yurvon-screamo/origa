import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { awaitHandVisible } from "../../helpers/lesson";

When("пользователь открывает урок напрямую", async ({ page }) => {
	// Транзит /words → /home редиректит в wizard (app-level, KNOWN_FIXME):
	// страница /lesson сама собирает руку, поэтому входим напрямую.
	const diagnostics = attachPageDiagnostics(page, "direct-lesson");
	try {
		await page.goto("/lesson");
		await page.waitForLoadState("domcontentloaded");
	} catch (err) {
		console.info(
			`[counter-diagnostics:direct-lesson-failure] url=${page.url()}\n${diagnostics()}`,
		);
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

When("пользователь начинает урок со счётными суффиксами", async ({ page }) => {
	const diagnostics = attachPageDiagnostics(page, "start-lesson");
	try {
		const homePage = new (await import("../../pages")).HomePage(page);
		await homePage.goto();
		await homePage.startLesson();
	} catch (err) {
		console.info(
			`[counter-diagnostics:start-lesson-failure] url=${page.url()}\n${diagnostics()}`,
		);
		throw err;
	}
});

/// Показ руки: жмём «Дальше», пока не встретим слайд счётного суффикса
/// (issue #415: миграция заводит counter-карту из слова «一本»; в маленьком
/// пуле рука содержит и слово, и счётчик — слайд гарантированно в показе).
Then("в показе руки знакомства нет слайда счётного суффикса", async ({ page }) => {
	await awaitHandVisible(page);
	const counterSlide = page.getByTestId("acquaintance-counter-slide");
	const nextBtn = page.getByTestId("acquaintance-next-btn");
	// Проходим показ до конца: контр-слайда не должно встретиться ни на одном
	// слайде (пул маленький — 1–2 слова, показ короткий).
	for (let i = 0; i < 20; i++) {
		expect(
			await counterSlide.isVisible().catch(() => false),
			"counter slide must not appear for prose words",
		).toBe(false);
		await nextBtn.click({ timeout: 3_000 }).catch(() => null);
		const training = await page
			.getByTestId("acquaintance-training")
			.waitFor({ state: "visible", timeout: 500 })
			.then(() => true)
			.catch(() => false);
		if (training) break;
	}
	expect(await counterSlide.isVisible().catch(() => false)).toBe(false);
});

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
