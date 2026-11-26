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
  // The static splash fades for 360 ms after WASM mounts; it has an id,
  // not a test id, so this is the one sanctioned CSS selector here.
  await page
    .locator("#origa-boot-splash")
    .waitFor({ state: "detached", timeout: 10_000 })
    .catch(() => undefined);
  await sink.shot(screen);
}
