import { expect } from "@playwright/test";
import { When, Then, Given } from "../fixtures";
import { awaitHandVisible } from "../../helpers/lesson";
import {
	countReviewedCounterBindings,
	readPersistedUser,
	reviewedCounterBindingNumbers,
} from "../../helpers/indexeddb";

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

When('пользователь раскрывает таблицу чтений на ответе связки', async ({ page }) => {
	await page.keyboard.press(" ");
	await page.getByTestId("counter-binding-answer").waitFor({ timeout: 10_000 });
	// Таблица свёрнута по умолчанию (expand/collapse-паттерн объёмных
	// ответов): проверяем скрытость, раскрываем, после — обратный сворот.
	await expect(
		page.getByTestId("counter-binding-mutations"),
	).toHaveCount(0);
	await page.getByTestId("counter-binding-mutations-toggle").click();
	const table = page.getByTestId("counter-binding-mutations-table");
	await table.waitFor({ timeout: 10_000 });
	// Строки двух testid: обычная и акцентная (отвеченная) строка.
	const rows = table.locator(
		'[data-testid="counter-mutations-row"], [data-testid="counter-readings-row-answered"]',
	);
	expect(await rows.count()).toBeGreaterThanOrEqual(11);
	// Акцент отвеченной строки — data-атрибут (независим от стилей).
	const highlighted = table.getByTestId("counter-readings-row-answered");
	expect(
		await highlighted.count(),
		"the answered row must be accented",
	).toBeGreaterThanOrEqual(1);
	// Обратный сворот прячет таблицу.
	await page.getByTestId("counter-binding-mutations-toggle").click();
	await expect(page.getByTestId("counter-binding-mutations")).toHaveCount(0);
});

When('пользователь жмёт {string} отвечая {string}', async ({ page }, key: string, _answer: string) => {
	// Рейтинг — ТОЛЬКО хоткей: у слота своя клавиатура, клики по кнопкам
	// здесь не нужны (нет интерференции с общим обработчиком урока).
	// Самодостаточность: дождаться слота (WASM-гидрация может длиться
	// секунды), затем если ответ не раскрыт — сначала пробел.
	await page.getByTestId("counter-bindings-card").waitFor({ timeout: 30_000 });
	const front = page.getByTestId("counter-binding-front");
	if (await front.isVisible({ timeout: 5_000 }).catch(() => false)) {
		await page.keyboard.press(" ");
		await page.getByTestId("counter-binding-answer").waitFor({ timeout: 10_000 });
	}
	const progress = page.getByTestId("counter-bindings-progress");
	const before = await progress
		.textContent({ timeout: 2_000 })
		.catch(() => null);
	// Сколько связок уже несут память — база для ожидания записи ниже.
	const reviewedBefore = countReviewedCounterBindings(
		await readPersistedUser(page).catch(() => null),
	);
	await page.keyboard.press(key);
	// Пачка переходит локально мгновенно, а запись оценки асинхронна
	// (spawn_local → save): дожидаемся перехода по прогрессу и флеша
	// сохранения, иначе быстрый goto перезахода проигрывает запись.
	// Последняя связка пачки завершает слот — прогресс исчезает.
	if (before === null) {
		return;
	}
	await expect
		.poll(
			async () =>
				await progress.textContent({ timeout: 2_000 }).catch(() => null),
			{ timeout: 10_000 },
		)
		.not.toBe(before);
	// Флеш сохранения: оценка связки уходит в локальное хранилище
	// асинхронно (spawn_local → save), а следующий шаг перезаходит в урок
	// (полная перезагрузка). Ждём СОСТОЯНИЕ хранилища: у отвеченной
	// связки появляется память (у новой связки её не было). Семантическая
	// память карты тут не показатель — рейтинг связки её не трогает.
	await expect
		.poll(
			async () =>
				countReviewedCounterBindings(
					await readPersistedUser(page).catch(() => null),
				),
			// The store row is read as a whole — poll it twice a second,
			// not ten times.
			{ timeout: 15_000, intervals: [500] },
		)
		.toBeGreaterThan(reviewedBefore);
});

Then('пачка переходит к следующей цифре', async ({ page }) => {
	const progress = page.getByTestId("counter-bindings-progress");
	await expect
		.poll(async () => progress.textContent(), { timeout: 15_000 })
		.toContain("2 / 11");
});

When('пользователь раскрывает ответ связки', async ({ page }) => {
	await page.getByTestId("counter-bindings-card").waitFor({ timeout: 30_000 });
	await page.getByTestId("counter-binding-front").waitFor({ timeout: 10_000 });
	await page.keyboard.press(" ");
	await page.getByTestId("counter-binding-answer").waitFor({ timeout: 10_000 });
});

Then('на ответе связки есть фуригана с чтением из реестра', async ({ page }) => {
	// Фуригана — ruby-подпись чтения из реестра (не по кускам: 1日 =
	// ついたчи, а не いち+にчи).
	const rt = page.getByTestId("counter-binding-answer-reading");
	await rt.waitFor({ timeout: 10_000 });
	const reading = ((await rt.textContent()) ?? "").trim();
	expect(reading.length).toBeGreaterThan(0);
	// Чтение каной (хирагана) — из реестра счётных суффиксов.
	expect(reading).toMatch(/[\u3040-\u309f]/);
});

Then('на ответе связки есть кнопка озвучки', async ({ page }) => {
	await page.getByTestId("counter-binding-audio-btn").waitFor({ timeout: 10_000 });
});

When('пользователь проходит пачку связок до конца', async ({ page }) => {
	await page.getByTestId("counter-bindings-card").waitFor({ timeout: 30_000 });
	await page.getByTestId("counter-binding-front").waitFor({ timeout: 10_000 });
	// Полный проход: Space раскрывает, «2» отвечает «Знаю». Последняя
	// связка (何) замораживается; advance сериализован после сейва —
	// потому ждём экран завершения терпеливо, лишние нажатия гасятся
	// finished-гейтом сессии.
	// Граница цикла: 11 связок реестра (1..=10 + 何) + запас на лишние
	// нажатия. Если в пачку попадут beyond-ten формы — границу поднять.
	for (let i = 0; i < 15; i++) {
		if (await page.getByTestId("lesson-complete-screen").isVisible().catch(() => false)) {
			break;
		}
		await page.keyboard.press(" ");
		await page.getByTestId("counter-binding-answer").waitFor({ timeout: 10_000 });
		const progressBefore = await page
			.getByTestId("counter-bindings-progress")
			.textContent({ timeout: 2_000 })
			.catch(() => null);
		await page.keyboard.press("2");
		if (progressBefore === null) {
			// Счётчика прогресса не было — ждать нечего.
			continue;
		}
		// Сессия переходит к следующей связке: растёт счётчик прогресса.
		// Последняя связка закрывает пачку — элемент прогресса исчезает,
		// и это тоже переход.
		await expect
			.poll(
				async () => {
					const advanced = await page
						.getByTestId("counter-bindings-progress")
						.textContent({ timeout: 2_000 })
						.catch(() => null);
					return advanced !== progressBefore;
				},
				{ timeout: 15_000 },
			)
			.toBe(true);
	}
});

Then('урок завершается экраном прохождения', async ({ page }) => {
	await page.getByTestId("lesson-complete-screen").waitFor({ timeout: 20_000 });
});

When('пользователь перезаходит в урок', async ({ page }) => {
	await page.goto("http://localhost:1420/lesson");
});

Then('отвеченная связка больше не показывается в пачке', async ({ page }) => {
	await page.getByTestId("counter-bindings-card").waitFor({ timeout: 30_000 });
	await page.getByTestId("counter-binding-front").waitFor({ timeout: 10_000 });
	// Порядок пачки случайный, а window не переживает перезагрузку
	// (следующий шаг делает goto) — источник истины: связки с памятью в
	// IDB (оценённая связка получила current_state).
	const rated = reviewedCounterBindingNumbers(await readPersistedUser(page));
	expect(rated.length, "должна быть ровно одна оценённая связка").toBe(1);
	await expect
		.poll(
			async () =>
				page.getByTestId("counter-binding-front").locator("span").first().textContent(),
			{ timeout: 15_000 },
		)
		.not.toBe(String(rated[0]));
	const progress = await page.getByTestId("counter-bindings-progress").textContent();
	expect(progress).toContain("1 / 10");
});

When('пользователь жмёт пробел для ответа первой связки', async ({ page }) => {
	await page.getByTestId("counter-bindings-card").waitFor({ timeout: 30_000 });
	await page.keyboard.press(" ");
	await page.getByTestId("counter-binding-answer").waitFor({ timeout: 10_000 });
});
