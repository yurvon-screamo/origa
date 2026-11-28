import { expect, type Page } from "@playwright/test";

import type { ShotSink } from "../types.ts";

/** The static splash has an id, not a test id — the one sanctioned CSS selector. */
export async function waitForAppReady(page: Page): Promise<void> {
  await page
    .locator("#origa-boot-splash")
    .waitFor({ state: "detached", timeout: 10_000 })
    .catch(() => undefined);
}

/**
 * The home mount sync is asynchronous: first give the toast a window to
 * mount, then wait until it is gone. A toast caught mid-fade is a false
 * diff on the next PR. One wait for every home screenshot.
 */
export async function waitForSyncToastToDisappear(page: Page): Promise<void> {
  const toast = page.locator('[data-testid^="toast-"]').first();
  await toast
    .waitFor({ state: "visible", timeout: 2_000 })
    .catch(() => undefined);
  await expect(page.locator('[data-testid^="toast-"]')).toHaveCount(0, {
    timeout: 15_000,
  });
}

/** Captures one marker-driven screen of a simple page. */
export async function captureSimplePage(
  page: Page,
  sink: ShotSink,
  route: string,
  marker: string,
  screen: string,
): Promise<void> {
  await page.goto(route);
  await expect(page.getByTestId(marker)).toBeVisible({ timeout: 60_000 });
  await waitForAppReady(page);
  await sink.shot(screen);
}
