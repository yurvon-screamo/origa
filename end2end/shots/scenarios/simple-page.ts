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
 * The home mount sync is asynchronous: give the toast a short window to
 * appear before asserting it has gone. One wait for every home screenshot —
 * a toast caught mid-fade is a false diff on the next PR.
 */
export async function waitForSyncToast(page: Page): Promise<void> {
  await page.waitForTimeout(300);
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
