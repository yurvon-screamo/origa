import { LessonPage } from "../../pages/lesson.page.ts";
import { WordsPage } from "../../pages/words.page.ts";
import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The feedback channel of the lesson: the report button in the lesson
 * header, the opened modal and the token popup with its report line.
 *
 * The profile behind the screens has one word, so the lesson has a hand to
 * show.
 */
export const feedbackFormScenario: ShotScenario = {
    ...descriptors["feedback-form"],
    async run({ page }, sink) {
        // Add a word from text — the same flow the BDD scenarios use.
        await page.goto("/words");
        const words = new WordsPage(page);
        await words.wordsPage.waitFor({ timeout: 30_000 });
        await words.addButton.click();
        await words.drawerTextarea.fill("私は本を読みます");
        await words.drawerAnalyzeBtn.click();
        await page.getByText(/Найдено/).waitFor({ timeout: 15_000 });
        const items = words.analyzedWordItems;
        await items.first().waitFor({ state: "visible", timeout: 10_000 });
        // analyze_text() pre-selects everything; deselect all, then pick the
        // first word (including the disabled ones, they are skipped).
        const count = await items.count();
        for (let index = 0; index < count; index++) {
            const item = items.nth(index);
            const checkbox = item.locator('input[type="checkbox"]');
            if (await checkbox.isChecked().catch(() => false)) {
                const isDisabled = await item
                    .evaluate((element) => element.classList.contains("cursor-not-allowed"))
                    .catch(() => false);
                if (isDisabled) continue;
                await item.click();
                await page.waitForTimeout(150);
            }
        }
        await items.first().click();
        await page.waitForTimeout(200);
        await words.drawerAddBtn.click();
        await words.wordsGrid.waitFor({ timeout: 20_000 });

        // Start a lesson.
        await page.goto("/lesson");
        await page.getByTestId("lesson-header").waitFor({ state: "visible", timeout: 60_000 });
        const lesson = new LessonPage(page);
        await lesson.expectLessonVisible();

        // 1) Lesson header with the report button (question phase: disabled).
        await page.getByTestId("lesson-report-btn").waitFor({ state: "visible", timeout: 60_000 });
        await sink.shot("lesson-header-report-disabled");

        // Walk the acquaintance presentation to the training reveal.
        await page.getByTestId("acquaintance-view").waitFor({ state: "visible", timeout: 30_000 });
        const next = page.getByTestId("acquaintance-next-btn");
        for (let index = 0; index < 20; index++) {
            await next.click({ timeout: 3_000 }).catch(() => undefined);
            const training = await page
                .getByTestId("acquaintance-training")
                .waitFor({ state: "visible", timeout: 1_000 })
                .then(() => true)
                .catch(() => false);
            if (training) break;
        }
        await page.getByTestId("acquaintance-training").waitFor({ state: "visible", timeout: 15_000 });
        const reveal = page.getByTestId("acquaintance-reveal-btn");
        await reveal.waitFor({ state: "visible", timeout: 30_000 });
        await reveal.click();
        await page.waitForTimeout(500);

        // 2) The feedback modal opened from the lesson header.
        await page.getByTestId("lesson-report-btn").click();
        await page.getByTestId("feedback-modal").waitFor({ state: "visible", timeout: 10_000 });
        await sink.shot("feedback-modal-subject");

        // 3) The same modal with a filled message.
        await page.getByTestId("feedback-message-input").fill("Чтение не соответствует карточке");
        await sink.shot("feedback-modal-filled");

        // 4) The token popup with the report line: the phrases page renders
        // a translator text on every phrase card.
        await page.keyboard.press("Escape");
        await page.waitForTimeout(400);
        await page.goto("/phrases");
        await page.getByTestId("phrases-card-item").first().waitFor({ state: "visible", timeout: 30_000 });
        await page.waitForTimeout(3_000);
        const token = page.locator(".token-word .token-surface").first();
        await token.waitFor({ state: "visible", timeout: 30_000 });
        await token.click();
        await page.getByTestId("token-popup-report").waitFor({ state: "visible", timeout: 10_000 });
        await sink.shot("token-popup-report");
    },
};
