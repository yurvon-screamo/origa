import { expect, type Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";
import { clickFirstVisible } from "./review-body.ts";

const OPTIONS = [
  "grammar-practice-option-0",
  "grammar-practice-option-1",
  "grammar-practice-option-2",
  "grammar-practice-option-3",
];
const DELETE_BUTTONS = [
  "grammar-detail-actions-delete-btn",
  "grammar-detail-actions-mobile-delete-btn",
];
const MAX_ANSWERS = 30;

/** Brings a marker to the top of the frame before the screenshot. */
async function framePractice(page: Page, testId: string): Promise<void> {
  const marker = page.getByTestId(testId);
  await marker.evaluate((element) => {
    const top = element.getBoundingClientRect().top + window.scrollY - 24;
    window.scrollTo({ top });
  });
  await expect(marker).toBeInViewport();
}

/** The answer frame waits for the options to reach their final colours. */
async function optionsSettled(page: Page): Promise<void> {
  await expect
    .poll(
      async () =>
        page
          .locator('[data-testid^="grammar-practice-option-"]')
          .evaluateAll((elements) =>
            elements.every((element) => {
              const style = getComputedStyle(element);
              if (element.classList.contains("quiz-option-dimmed")) {
                return Math.abs(Number(style.opacity) - 0.5) < 0.01;
              }
              if (
                element.classList.contains("quiz-option-correct") ||
                element.classList.contains("quiz-option-wrong")
              ) {
                const variable = element.classList.contains(
                  "quiz-option-correct",
                )
                  ? "--success"
                  : "--error";
                const hex = getComputedStyle(document.documentElement)
                  .getPropertyValue(variable)
                  .trim();
                const expected = `rgb(${parseInt(hex.slice(1, 3), 16)}, ${parseInt(hex.slice(3, 5), 16)}, ${parseInt(hex.slice(5, 7), 16)})`;
                return style.color === expected;
              }
              return false;
            }),
          ),
      { timeout: 5_000 },
    )
    .toBe(true);
}

/** The question frame waits for the mobile tab underline to stop moving. */
async function tabsSettled(page: Page): Promise<void> {
  await expect
    .poll(
      async () =>
        page.evaluate(() => {
          const scale = (testId: string) => {
            const element = document.querySelector(
              '[data-testid="' + testId + '"]',
            );
            if (!element) return null;
            const transform = getComputedStyle(element, "::after").transform;
            const matrix = transform.match(/matrix\(([^)]+)\)/);
            return matrix ? Number(matrix[1].split(",")[0]) : null;
          };
          const active = scale("grammar-detail-tabs-practice");
          const inactive = scale("grammar-detail-tabs-overview");
          return (
            active !== null &&
            inactive !== null &&
            Math.abs(active - 1) < 0.01 &&
            Math.abs(inactive) < 0.01
          );
        }),
      { timeout: 5_000 },
    )
    .toBe(true);
}

/**
 * A grammar rule detail: overview, the practice session and the delete
 * confirmation. The practice preset seeds a quiz-capable rule plus one
 * known verb — the same pair the BDD grammar steps use, without the
 * minutes of UI seeding on every viewport.
 */
export const grammarDetailScenario: ShotScenario = {
  ...descriptors["grammar-detail"],
  async run({ page }, sink) {
    await page.goto("/grammar");
    await expect(page.getByTestId("grammar-card-item").first()).toBeVisible({
      timeout: 60_000,
    });
    await page.getByTestId("grammar-card-item").first().click();
    await expect(page.getByTestId("grammar-detail-container")).toBeVisible({
      timeout: 30_000,
    });
    await expect(page.getByTestId("grammar-detail-breadcrumbs")).toBeVisible({
      timeout: 30_000,
    });
    await sink.shot("grammar-detail-overview");

    const practiceTab = page.getByTestId("grammar-detail-tabs-practice");
    if (await practiceTab.isVisible().catch(() => false)) {
      await practiceTab.click();
      await tabsSettled(page);
    }
    await expect(
      page
        .getByTestId("grammar-practice-progress")
        .or(page.getByTestId("grammar-practice-complete"))
        .or(page.getByTestId("grammar-practice-no-words")),
    ).toBeVisible({ timeout: 15_000 });
    if (
      await page
        .getByTestId("grammar-practice-complete")
        .isVisible()
        .catch(() => false)
    ) {
      await page.getByTestId("grammar-practice-again-btn").click();
    }
    await expect(page.getByTestId("grammar-practice-progress")).toBeVisible({
      timeout: 15_000,
    });
    await framePractice(page, "grammar-practice-progress");
    await sink.shot("grammar-practice-question");

    await clickFirstVisible(page, OPTIONS);
    await optionsSettled(page);
    const nextButton = page.getByTestId("grammar-practice-next-btn");
    await expect(nextButton).toBeVisible({ timeout: 10_000 });
    await framePractice(page, "grammar-practice-progress");
    await sink.shot("grammar-practice-answer");

    const complete = page.getByTestId("grammar-practice-complete");
    for (let answered = 0; answered < MAX_ANSWERS; answered++) {
      if (await complete.isVisible()) break;
      if (await nextButton.isVisible()) {
        await nextButton.click({ timeout: 5_000 });
      } else if (!(await clickFirstVisible(page, OPTIONS))) {
        await expect(nextButton).toBeVisible({ timeout: 10_000 });
      }
    }
    await expect(complete).toBeVisible({ timeout: 10_000 });
    await framePractice(page, "grammar-practice-complete");
    await sink.shot("grammar-practice-complete");

    await clickFirstVisible(page, DELETE_BUTTONS);
    const modal = page.getByTestId("grammar-detail-delete-modal");
    await expect(modal).toBeVisible({ timeout: 10_000 });
    await expect
      .poll(
        async () =>
          modal.evaluate((element) =>
            Number(getComputedStyle(element).opacity),
          ),
        { timeout: 5_000 },
      )
      .toBeGreaterThan(0.95);
    await sink.shot("grammar-delete-modal");
  },
};
