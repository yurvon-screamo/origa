import { expect, type Page } from "@playwright/test";

import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The kanji-detail writing practice (stroke-count indicator + canvas).
 *
 * The bug this screen documents: the corner stroke-progress label used
 * to pin to the full-width container edge (leaving the viewport on
 * narrow layouts) instead of the drawing box. The capture lands
 * directly on the seeded kanji card's detail (acquaintance-hand —
 * seconds to prepare, no corpus import): the desktop grid shows the
 * practice in the right column, the phone layout needs the
 * «НАПИСАНИЕ» tab before the canvas mounts.
 */
export const kanjiWritingPracticeScenario: ShotScenario = {
  ...descriptors["kanji-writing-practice"],
  async run({ page, viewport }, sink) {
    // The acquaintance-hand seed gives the kanji card a stable ULid.
    await page.goto("/kanji/01JCAV0KQVYD4F5H8P0EQ5Z792", {
      waitUntil: "domcontentloaded",
    });

    // Both layout branches exist in the DOM (the media query hides the
    // off-branch one), so every locator scopes to THIS viewport's branch.
    // The phone layout gates the practice behind the «НАПИСАНИЕ» tab —
    // the tab click must come before the canvas wait (the canvas mounts
    // only then).
    const canvasSelector =
      viewport === "mobile"
        ? ".kanji-detail-mobile canvas.kanji-drawing-canvas"
        : ".kanji-detail-desktop canvas.kanji-drawing-canvas";

    if (viewport === "mobile") {
      await page
        .getByTestId("kanji-detail-tabs-writing")
        .waitFor({ state: "visible", timeout: 120_000 });
      await page.getByTestId("kanji-detail-tabs-writing").click();
    }

    const canvasReady = await page
      .locator(canvasSelector)
      .waitFor({ state: "visible", timeout: 120_000 })
      .then(() => true)
      .catch(() => false);
    if (!canvasReady) {
      const dump = await page.evaluate(() => {
        const canvas = document.querySelector("canvas.kanji-drawing-canvas");
        const chain: string[] = [];
        let el: Element | null = canvas;
        while (el && el !== document.body) {
          const s = getComputedStyle(el);
          chain.push(
            `${el.tagName.toLowerCase()}.${(el.className + "").split(" ").slice(0, 2).join(".")}` +
              ` d=${s.display} v=${s.visibility} w=${Math.round(el.getBoundingClientRect().width)}`,
          );
          el = el.parentElement;
        }
        return {
          url: location.pathname,
          canvasFound: !!canvas,
          chain,
          testids: Array.from(document.querySelectorAll("[data-testid]"))
            .map((e) => e.getAttribute("data-testid"))
            .filter((x) => /kanji|drawing|loading|tab/i.test(x ?? ""))
            .slice(0, 15),
        };
      });
      console.log("[shots] canvas wait failed:", JSON.stringify(dump, null, 1));
      expect(
        canvasReady,
        "the drawing canvas must mount on the detail page",
      ).toBe(true);
    }

    // Let the kanji art SVG and the corner label settle before the shot.
    await page.waitForTimeout(700);
    await sink.shot("kanji-writing-practice", { fullPage: true });
  },
};
