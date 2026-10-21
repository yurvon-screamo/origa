import { LessonPage } from "../../pages/lesson.page.ts";
import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The feedback channel of the lesson: the report button in the lesson
 * header, the opened modal and the token popup with its report line.
 *
 * The profile behind the screens has one word, so the lesson has a hand to
 * show (see the `with-word` preset).
 */
export const feedbackFormScenario: ShotScenario = {
    ...descriptors["feedback-form"],
    async run({ page }, sink) {
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
    },
};
