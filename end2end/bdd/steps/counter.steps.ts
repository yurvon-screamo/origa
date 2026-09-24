import { expect } from "@playwright/test";
import { When, Then, Given } from "../fixtures";
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

/// Подготовка состояния «пачка готова»: сид руки с прошедшей датой.
/// КОНТРАКТ: инъекция завязана на persisted-схему HybridUserRepository
/// (db=origa, store=users, вложенная JSON-строка) — при рефакторинге
/// хранилища шаг требует синхронной правки.
/// (семантика due, связки-новички) — «следующий день» без перемотки часов.
Given('связки счётного суффикса готовы к показу', async ({ page }) => {
	await page.evaluate(async () => {
		const past = "2020-01-01T00:00:00Z";
		const state = {
			stability: { value: 3.0 },
			difficulty: { value: 5.0 },
			next_review_date: past,
			card_state: "Review",
		};
		const db = await new Promise<IDBDatabase>((res) => {
			const r = indexedDB.open("origa");
			r.onsuccess = () => res(r.result);
		});
		await new Promise<void>((res) => {
			const tx = db.transaction("users", "readwrite");
			const cur = tx.objectStore("users").openCursor();
			cur.onsuccess = () => {
				const c = cur.result;
				if (!c) return;
				const v = c.value;
				if (typeof v === "string" && v.includes('"email"')) {
					try {
						const u = JSON.parse(v);
						// Оставляем ТОЛЬКО counter-карты: слово-новичок (一)
						// открыл бы руку знакомства раньше основного урока.
						const keep: Record<string, unknown> = {};
						for (const [id, card] of Object.entries(
							u.knowledge_set?.study_cards ?? {},
						) as [string, Record<string, unknown>][]) {
							if ((card as { card?: { Counter?: unknown } }).card?.Counter) {
								card.memory_history = {
									current_state: state,
									reps: 1,
									lapses: 0,
									easy_count: 0,
									good_count: 1,
									last_review_date: past,
									last_rating: "Good",
									consecutive_again: 0,
								};
								keep[id] = card;
							}
						}
						u.knowledge_set.study_cards = keep;
						c.update(JSON.stringify(u));
					} catch { /* skip */ }
				}
				c.continue();
			};
			tx.oncomplete = () => { db.close(); res(); };
		});
	});
});

Then('в уроке показывается пачка связок счётного суффикса', async ({ page }) => {
	await page.getByTestId("counter-bindings-card").waitFor({ timeout: 30_000 });
	await page.getByTestId("counter-binding-front").waitFor({ timeout: 10_000 });
});

When('пользователь раскрывает ответ первой связки', async ({ page }) => {
	await page.getByTestId("counter-binding-show-answer-btn").click();
	await page.getByTestId("counter-binding-answer").waitFor({ timeout: 10_000 });
});

Then('в ответе видна таблица чтений с акцентом строки', async ({ page }) => {
	const table = page.getByTestId("counter-binding-mutations-table");
	await table.waitFor({ timeout: 10_000 });
	const rows = table.getByTestId("counter-mutations-row");
	expect(await rows.count()).toBeGreaterThanOrEqual(11);
	// Акцент отвеченной строки — data-атрибут (независим от стилей).
	const highlighted = table.getByTestId("counter-readings-row-answered");
	expect(
		await highlighted.count(),
		"the answered row must be accented",
	).toBeGreaterThanOrEqual(1);
});

When('пользователь отвечает {string} на связку', async ({ page }, answer: string) => {
	await page.getByTestId("counter-binding-show-answer-btn").click().catch(() => null);
	await page.getByTestId("counter-binding-answer").waitFor({ timeout: 10_000 });
	const btn =
		answer === "Знаю"
			? page.getByTestId("lesson-rating-btn-good")
			: page.getByTestId("lesson-rating-btn-again");
	await btn.click();
	await page.waitForTimeout(500);
});

Then('пачка переходит к следующей цифре', async ({ page }) => {
	const progress = page.getByTestId("counter-bindings-progress");
	await expect(progress).toHaveText("2 / 11", { timeout: 10_000 });
});

When('пользователь перезаходит в урок', async ({ page }) => {
	await page.goto("http://localhost:1420/lesson");
});

Then('отвеченная связка больше не показывается в пачке', async ({ page }) => {
	await page.getByTestId("counter-bindings-card").waitFor({ timeout: 30_000 });
	await page.getByTestId("counter-binding-front").waitFor({ timeout: 10_000 });
	const front = await page.getByTestId("counter-binding-front").textContent();
	expect(front, "the rated-Good binding leaves the rotation").not.toContain("1×");
	const progress = await page.getByTestId("counter-bindings-progress").textContent();
	expect(progress).toContain("1 / 10");
});

When('пользователь жмёт пробел для ответа первой связки', async ({ page }) => {
	await page.getByTestId("counter-bindings-card").waitFor({ timeout: 30_000 });
	await page.keyboard.press(" ");
	await page.getByTestId("counter-binding-answer").waitFor({ timeout: 10_000 });
});

When('пользователь жмёт клавишу {string} отвечая {string}', async ({ page }, key: string, _answer: string) => {
	await page.keyboard.press(key);
	await page.waitForTimeout(400);
});
