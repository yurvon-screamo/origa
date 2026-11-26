import { expect, type Page } from "@playwright/test";

import type { ShotSink } from "../types.ts";

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
  await sink.shot(screen);
}
