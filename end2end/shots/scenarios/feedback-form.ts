import { expect } from "@playwright/test";

import { LessonPage } from "../../pages/lesson.page.ts";
import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { waitForAppReady } from "./simple-page.ts";

/**
 * The feedback channel of the lesson: the report button in the lesson
 * header, the opened modal and its filled state.
 *
 * The `due-corpus` preset opens the lesson straight into the review
 * body. The earlier `with-word` hand had to be walked to the reveal
 * before the modal would open; with a due queue the report entry is
 * reachable in the card phase the screen documents.
 */
export const feedbackFormScenario: ShotScenario = {
  ...descriptors["feedback-form"],
  async run({ page }, sink) {
    await page.goto("/lesson");
    await page
      .getByTestId("lesson-header")
      .waitFor({ state: "visible", timeout: 60_000 });
    const lesson = new LessonPage(page);
    await lesson.expectLessonVisible();
    await waitForAppReady(page);

    // 1) Lesson header with the report button (question phase).
    await page
      .getByTestId("lesson-report-btn")
      .waitFor({ state: "visible", timeout: 60_000 });
    await sink.shot("lesson-header-report-disabled", {
      marker: "lesson-report-btn",
    });

    // 2) The modal opened from the lesson header.
    await page.getByTestId("lesson-report-btn").click();
    await expect(page.getByTestId("feedback-modal")).toBeVisible({
      timeout: 15_000,
    });
    await sink.shot("feedback-modal-subject", { marker: "feedback-modal" });

    // 3) The same modal with a filled message.
    await page
      .getByTestId("feedback-message-input")
      .fill("Чтение не соответствует карточке");
    await sink.shot("feedback-modal-filled", {
      marker: "feedback-message-input",
    });
  },
};
