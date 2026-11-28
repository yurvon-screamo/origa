import type { BrowserContext, Page } from "@playwright/test";

import type { ShotEntry } from "./manifest.ts";
import type { ViewportTag } from "./viewports.ts";

/** What a scenario gets to work with. */
export interface ShotContext {
  page: Page;
  context: BrowserContext;
  baseURL: string;
  locale: string;
  viewport: ViewportTag;
}

/**
 * Where a scenario puts its screens.
 *
 * The sink owns the file writing and the bookkeeping (hash, size,
 * provenance) — a scenario only names the screen it wants to keep.
 */
export interface ShotSink {
  /**
   * `marker` is a test-id, or a `css=` selector where the markup has no
   * marker. The sink settles it before writing the frame.
   */
  shot(
    name: string,
    options?: { fullPage?: boolean; marker?: string },
  ): Promise<ShotEntry>;
  /** Everything captured through this sink, in capture order. */
  readonly entries: readonly ShotEntry[];
}
