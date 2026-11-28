import { writeFile } from "node:fs/promises";
import { join } from "node:path";
import { expect, type Page } from "@playwright/test";

import type { Examiner } from "./examiner/examine.ts";
import {
  readPngSize,
  sha256Hex,
  shotFileName,
  stampProof,
  type ShotEntry,
} from "./manifest.ts";
import { waitForAppReady } from "./scenarios/simple-page.ts";
import type { ShotSink } from "./types.ts";
import type { ViewportTag } from "./viewports.ts";

export interface SinkOptions {
  outDir: string;
  scenario: string;
  title: string;
  route: string;
  viewport: ViewportTag;
  locale: string;
  baseURL: string;
  cdnBaseUrl: string;
  commit: string;
  /** When set, every captured screen is also examined by the d3 decision model. */
  examiner?: Examiner;
}

const MARKER_TIMEOUT = 15_000;

interface EntryFacts {
  name: string;
  fullPage: boolean;
  file: string;
  width: number;
  height: number;
  bytes: number;
  sha256: string;
  capturedAt: string;
}

/**
 * The sink a scenario writes its screens into.
 *
 * It owns the file, the hash and the provenance: a scenario only says
 * which screen it wants and which test-id proves it. The marker is
 * settled before the pixels are written.
 */
export function createSink(page: Page, options: SinkOptions): ShotSink {
  const entries: ShotEntry[] = [];

  const baseEntry = (facts: EntryFacts): ShotEntry => ({
    scenario: options.scenario,
    title: options.title,
    screen: facts.name,
    route: options.route,
    viewport: options.viewport,
    locale: options.locale,
    width: facts.width,
    height: facts.height,
    fullPage: facts.fullPage,
    bytes: facts.bytes,
    sha256: facts.sha256,
    file: facts.file,
    capturedAt: facts.capturedAt,
    commit: options.commit,
    baseURL: options.baseURL,
    cdnBaseUrl: options.cdnBaseUrl,
  });

  return {
    entries,
    shot: async (name, shotOptions) => {
      const fullPage = shotOptions.fullPage ?? false;
      const marker = shotOptions.marker.startsWith("css=")
        ? page.locator(shotOptions.marker.slice(4)).first()
        : page.getByTestId(shotOptions.marker).first();
      const file = join(
        options.outDir,
        shotFileName(options.scenario, name, options.viewport, options.locale),
      );

      const settle = async (): Promise<boolean> => {
        await waitForAppReady(page);
        if ((await marker.count()) === 0) return false;
        return await marker.evaluate((element, allowOffscreen) => {
          const style = getComputedStyle(element);
          const rect = element.getBoundingClientRect();
          const viewportHeight = window.innerHeight;
          const visible = Math.max(
            0,
            Math.min(rect.bottom, viewportHeight) - Math.max(rect.top, 0),
          );
          const threshold = 0.8 * Math.min(rect.height, viewportHeight);
          return (
            style.visibility !== "hidden" &&
            style.display !== "none" &&
            Number(style.opacity) >= 0.95 &&
            (allowOffscreen || (rect.height > 0 && visible >= threshold))
          );
        }, fullPage);
      };

      let okay = true;
      try {
        await expect.poll(settle, { timeout: MARKER_TIMEOUT }).toBe(true);
      } catch {
        okay = false;
      }
      if (!okay) {
        const failed = baseEntry({
          name,
          fullPage,
          file: "",
          width: 0,
          height: 0,
          bytes: 0,
          sha256: "",
          capturedAt: new Date().toISOString(),
        });
        entries.push(failed);
        console.error(
          `  ✗ ${options.scenario}/${name} [${options.viewport}]: marker "${shotOptions.marker}" did not settle`,
        );
        return failed;
      }

      const capturedAt = new Date().toISOString();
      const captured = await page.screenshot({
        animations: "disabled",
        fullPage,
      });
      const stamped = stampProof(captured, {
        schema: 1,
        commit: options.commit,
        screen: name,
      });
      await writeFile(file, stamped);
      const size = readPngSize(stamped);
      const entry = baseEntry({
        name,
        fullPage,
        file,
        width: size.width,
        height: size.height,
        bytes: stamped.byteLength,
        sha256: sha256Hex(stamped),
        capturedAt,
      });
      entries.push(entry);
      // The exam never breaks the capture: examineScreen records
      // its own failures and returns null for unexamined screens.
      if (options.examiner) {
        await options.examiner.examineScreen(page, entry);
      }
      return entry;
    },
  };
}
