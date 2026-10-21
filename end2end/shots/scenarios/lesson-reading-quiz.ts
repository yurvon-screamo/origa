import { expect, type Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The kanji reading quiz (multi-select) in the review body, answered
 * LENIENTLY on purpose (issue #822): the correct options are resolved
 * from the CDN kanji dictionary and a strict subset (≥50%, none wrong)
 * is selected, so the capture shows the «зачтено, но не все чтения»
 * message with the missed list and the kanji name box — the exact states
 * the issue is about.
 *
 * The `due-corpus` preset makes the kanji cards due (everything else is
 * pushed out), so the lesson body is kanji only. The review view of a
 * kanji is a RANDOM mix (1/5 reading quiz, 1/5 meaning quiz, 1/5 yesno,
 * 1/5 writing, 1/5 normal) and the writing branch cannot be advanced
 * past (the drawing practice has no skip), so the walk re-enters the
 * lesson — a reload regenerates every view for free — until a reading
 * quiz shows up.
 */
export const lessonReadingQuizScenario: ShotScenario = {
    ...descriptors["lesson-reading-quiz"],
    async run({ page }, sink) {
        let quizShot = false;
        for (let attempt = 0; attempt < 8 && !quizShot; attempt++) {
            await page.goto("/lesson");
            await page
                .getByTestId("lesson-page")
                .waitFor({ state: "visible", timeout: 90_000 });
            console.log(`[shots] attempt ${attempt}: walking`);

            for (let card = 0; card < 14; card++) {
                // The target: the multi-select reading quiz.
                if (
                    await page
                        .getByTestId("quiz-submit-btn")
                        .isVisible()
                        .catch(() => false)
                ) {
                    await captureReadingQuiz(page, sink);
                    quizShot = true;
                    break;
                }

                // Meaning quiz (single-select): answer, take the Next.
                const singleOption = page.getByTestId("quiz-option-0");
                if (await singleOption.isVisible().catch(() => false)) {
                    await singleOption.click({ timeout: 4_000 }).catch(() => undefined);
                    await page.waitForTimeout(600);
                    await page
                        .getByTestId("lesson-card-next-btn")
                        .click({ timeout: 4_000 })
                        .catch(() => undefined);
                    await page.waitForTimeout(400);
                    continue;
                }

                // Yes/no card.
                const yesno = page.getByTestId("yesno-no-btn");
                if (await yesno.isVisible().catch(() => false)) {
                    await yesno.click({ timeout: 4_000 }).catch(() => undefined);
                    await page.waitForTimeout(600);
                    await page
                        .getByTestId("lesson-card-next-btn")
                        .click({ timeout: 4_000 })
                        .catch(() => undefined);
                    await page.waitForTimeout(400);
                    continue;
                }

                // Audio recall.
                const audioReveal = page.getByTestId("audio-recall-reveal-btn");
                if (await audioReveal.isVisible().catch(() => false)) {
                    await audioReveal.click({ timeout: 4_000 }).catch(() => undefined);
                    await page.waitForTimeout(600);
                    await page
                        .getByTestId("audio-recall-know-btn")
                        .click({ timeout: 4_000 })
                        .catch(() => undefined);
                    await page.waitForTimeout(400);
                    continue;
                }

                // Normal card: reveal, rate «хорошо».
                const reveal = page.getByTestId("lesson-show-answer-btn");
                if (await reveal.isVisible().catch(() => false)) {
                    await reveal.click({ timeout: 4_000 }).catch(() => undefined);
                    await page.waitForTimeout(600);
                    await page
                        .getByTestId("lesson-rating-btn-again")
                        .click({ timeout: 4_000 })
                        .catch(() => undefined);
                    await page.waitForTimeout(400);
                    continue;
                }

                // Held on the feedback card — advance.
                const nextBtn = page.getByTestId("lesson-card-next-btn");
                if (await nextBtn.isVisible().catch(() => false)) {
                    await nextBtn.click({ timeout: 4_000 }).catch(() => undefined);
                    await page.waitForTimeout(400);
                    continue;
                }

                // The writing card (canvas drawing, no skip) — abandon the
                // attempt; the outer loop re-enters the lesson.
                if (await page.locator("canvas").isVisible().catch(() => false)) {
                    console.log(`[shots] attempt ${attempt} card ${card}: writing card, reroll`);
                    break;
                }

                await page.waitForTimeout(700);
            }
        }

        expect(quizShot, "a kanji reading quiz must appear within the reroll budget").toBe(true);
    },
};

interface KanjiEntry {
    kanji: string;
    on_readings: string[];
    kun_readings: string[];
}

let kanjiDict: KanjiEntry[] | null = null;

async function loadKanjiDict(page: Page): Promise<KanjiEntry[]> {
    if (kanjiDict) return kanjiDict;
    // The same corpus the bundle reads: the local server on :8080 when a
    // cdn/ checkout exists, the production CDN otherwise.
    for (const base of ["http://localhost:8080", "https://s3.origa.uwuwu.net"]) {
        const entries = await page.evaluate(async (url) => {
            try {
                const res = await fetch(url);
                if (!res.ok) return null;
                return (await res.json()).kanji as KanjiEntry[];
            } catch {
                return null;
            }
        }, `${base}/dictionary/kanji.json`);
        if (entries) {
            kanjiDict = entries;
            return entries;
        }
    }
    throw new Error("the kanji dictionary is not reachable from the page");
}

async function captureReadingQuiz(page: Page, sink: { shot(name: string): Promise<unknown> }) {
    const kanji = ((await page.locator(".display-text").first().textContent()) ?? "").trim();
    const dict = await loadKanjiDict(page);
    const entry = dict.find((k) => k.kanji === kanji);
    // The quiz options: the correct ones are this kanji's readings; the
    // distractors are readings of OTHER kanji, excluded from the target
    // set by the generator — so the dictionary lookup is exact.
    const readings = new Set(entry ? [...entry.on_readings, ...entry.kun_readings] : []);

    const count = await page.locator('[data-testid^="quiz-option-"]').count();
    const correctIdxs: number[] = [];
    for (let i = 0; i < count; i++) {
        const raw = (await page.getByTestId(`quiz-option-${i}`).textContent()) ?? "";
        const text = raw.replace(/[\s0-9]/g, "");
        if (readings.has(text)) correctIdxs.push(i);
    }

    // A strict subset with ≥50% of the correct readings → the lenient
    // pass: green «зачтено», the missed list, the kanji name box.
    const toSelect =
        correctIdxs.length >= 2
            ? correctIdxs.slice(0, Math.ceil(correctIdxs.length / 2))
            : correctIdxs;
    for (const idx of toSelect) {
        await page.getByTestId(`quiz-option-${idx}`).click({ timeout: 5_000 }).catch(() => {});
        await page.waitForTimeout(150);
    }

    await page.waitForTimeout(500);
    await sink.shot("quiz-question");

    await page.getByTestId("quiz-submit-btn").click({ timeout: 8_000 });
    await page
        .getByTestId("lesson-card-next-btn")
        .waitFor({ state: "visible", timeout: 20_000 })
        .catch(() => undefined);
    await page.waitForTimeout(700);
    await sink.shot("quiz-answer");

    await page
        .getByTestId("lesson-card-next-btn")
        .click({ timeout: 8_000 })
        .catch(() => undefined);
}
