import type { Page } from "@playwright/test";

import { ADVANCES, REVEALS, clickFirstVisible } from "./review-body.ts";

/**
 * Walks the real review body until `visit` finds its DOM discriminator.
 *
 * The lesson mixes card views at random. A blind reload reroll spends 30
 * seconds on every miss and never proves that the wanted view is
 * reachable through the app. This walk advances the actual card controls
 * instead, so each miss costs one card, not one page load.
 *
 * `visit` may capture the frames of the matched view. Returning true ends
 * the walk. A writing card (no reveal control) or the completion screen
 * ends it honestly; the caller rerolls the lesson.
 */
export async function walkLessonCards(
  page: Page,
  visit: (page: Page) => Promise<boolean>,
  maxCards = 8,
): Promise<boolean> {
  // The corpus boot can outlast the ten-second helper on a busy
  // machine; a walk that starts on the splash sees no cards at all.
  await page
    .locator("#origa-boot-splash")
    .waitFor({ state: "detached", timeout: 180_000 })
    .catch(() => undefined);
  for (let card = 0; card < maxCards; card++) {
    if (await visit(page)) return true;
    const complete = await page
      .getByTestId("lesson-complete-screen")
      .waitFor({ state: "visible", timeout: 500 })
      .then(() => true)
      .catch(() => false);
    if (complete) return false;
    const revealed = await clickFirstVisible(page, REVEALS, 1_500).catch(
      () => null,
    );
    if (!revealed) return false;
    const advanced = await clickFirstVisible(page, ADVANCES, 1_500).catch(
      () => null,
    );
    if (!advanced) return false;
  }
  return false;
}
