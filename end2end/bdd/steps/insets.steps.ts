import { join } from "path";
import { expect, type Page } from "@playwright/test";
import { When, Then } from "../fixtures";

/**
 * Steps for the edge-to-edge insets feature (task 20261109231215-gf1g0c1,
 * tablet branch: uwuwu#406).
 *
 * The device inset itself is 0 in a headless desktop browser, so the runtime
 * behaviour is pinned through the facts that a real device combines:
 * the shell class chosen per route (`pb-safe-b` vs `app-main-with-nav`),
 * the CSS rules actually present in the served stylesheet (the tailwind
 * utility declaring `env(safe-area-inset-bottom)` and the banner's
 * `env(safe-area-inset-top)` padding), and the computed reserve at >=1024px
 * where the bar is `lg:hidden` (driven by overriding `--safe-bottom`).
 * If a utility or the width branch is ever dropped from the generated CSS,
 * an assertion fails even though the class still renders "nothing" on desktop.
 */

async function shellInfo(page: Page): Promise<{
  hasSafeB: boolean;
  hasWithNav: boolean;
}> {
  const main = page.getByTestId("app-main");
  await expect(main).toBeVisible({ timeout: 15_000 });
  const info = await main.evaluate((el) => ({
    hasSafeB: el.classList.contains("pb-safe-b"),
    hasWithNav: el.classList.contains("app-main-with-nav"),
  }));
  return info;
}

Then("оболочка резервирует нижний inset устройства", async ({ page }) => {
  const info = await shellInfo(page);
  expect(
    info.hasSafeB,
    "the shell must reserve the device inset directly (pb-safe-b)",
  ).toBe(true);
});

Then("оболочка не резервирует клиренс нижней панели", async ({ page }) => {
  const info = await shellInfo(page);
  expect(
    info.hasWithNav,
    "the bar clearance must NOT be applied where the bar is hidden",
  ).toBe(false);
});

Then("оболочка резервирует клиренс нижней панели", async ({ page }) => {
  const info = await shellInfo(page);
  expect(
    info.hasWithNav,
    "the shell must reserve the bar clearance (#845)",
  ).toBe(true);
});

Then("оболочка не резервирует нижний inset устройства", async ({ page }) => {
  const info = await shellInfo(page);
  expect(
    info.hasSafeB,
    "the direct inset reserve must NOT be applied where the bar carries it",
  ).toBe(false);
});

When("пользователь устанавливает размер экрана 1024×768", async ({ page }) => {
  await page.setViewportSize({ width: 1024, height: 768 });
});

Then("нижняя панель приложения скрыта на широком экране", async ({ page }) => {
  // The nav is rendered by the route predicate but hidden by Tailwind's
  // `lg:hidden`: this is the precondition of the tablet reserve gap.
  await expect(page.locator(".bottom-tab-bar")).toBeHidden();
});

Then(
  "оболочка резервирует нижний inset на широком экране",
  async ({ page }) => {
    const main = page.getByTestId("app-main");
    await expect(main).toBeVisible({ timeout: 15_000 });
    // env() is 0 in a desktop browser: drive the shell through the same
    // :root variable the stylesheet uses, so the reserve is observable.
    await page.evaluate(() =>
      document.documentElement.style.setProperty("--safe-bottom", "48px"),
    );
    try {
      const shotDir = process.env.ORIGA_INSET_SHOT_DIR;
      if (shotDir) {
        // The reserve is felt at the end of a scrollable page: capture that
        // edge, where the system bar would otherwise cover the last content.
        await page.evaluate(() =>
          window.scrollTo(0, document.documentElement.scrollHeight),
        );
        await page.screenshot({
          path: join(shotDir, "tablet-home-bottom.png"),
        });
      }
      const paddingBottom = await main.evaluate((el) =>
        parseFloat(getComputedStyle(el).paddingBottom),
      );
      expect(
        paddingBottom,
        "with the bar hidden at >=1024px the shell must still reserve the device inset",
      ).toBe(48);
    } finally {
      await page.evaluate(() =>
        document.documentElement.style.removeProperty("--safe-bottom"),
      );
    }
  },
);

/**
 * Desktop boundary: with no device inset reported (env() = 0 in a headless
 * browser) the wide-viewport branch must stay pixel-identical to the old
 * `padding-bottom: 0` — the fix must not move content up on a plain monitor.
 * Also used after the injected step to pin its `finally` cleanup.
 */
async function expectNoWideReserve(page: Page, reason: string): Promise<void> {
  const main = page.getByTestId("app-main");
  await expect(main).toBeVisible({ timeout: 15_000 });
  const injected = await page.evaluate(() =>
    document.documentElement.style.getPropertyValue("--safe-bottom"),
  );
  expect(injected, `${reason}: the inset override must be absent`).toBe("");
  const paddingBottom = await main.evaluate((el) =>
    parseFloat(getComputedStyle(el).paddingBottom),
  );
  expect(
    paddingBottom,
    `${reason}: no device inset means no reserve at >=1024px`,
  ).toBe(0);
}

Then("на широком экране нет резерва без системного inset", async ({ page }) => {
  await expectNoWideReserve(page, "desktop baseline");
});

Then(
  "симуляция inset снята, резерв широкого экрана снова нулевой",
  async ({ page }) => {
    await expectNoWideReserve(page, "after simulation cleanup");
  },
);

/** Collects the cssText of rules whose selector matches `prefix`. */
async function cssRules(page: Page, prefix: string): Promise<string[]> {
  return await page.evaluate((prefix) => {
    const found: string[] = [];
    for (const sheet of Array.from(document.styleSheets)) {
      let list: CSSRuleList;
      try {
        list = sheet.cssRules;
      } catch {
        continue;
      }
      for (const rule of Array.from(list)) {
        if (rule.cssText.startsWith(prefix)) found.push(rule.cssText);
      }
    }
    return found;
  }, prefix);
}

Then(
  "правило резерва нижнего inset объявляет env safe-area-inset-bottom",
  async ({ page }) => {
    const rules = await cssRules(page, ".pb-safe-b");
    expect(
      rules,
      ".pb-safe-b rule must exist in the served CSS",
    ).not.toHaveLength(0);
    expect(
      rules.some((r) => r.includes("env(safe-area-inset-bottom")),
      `the rule must declare env(safe-area-inset-bottom), got: ${rules.join(" | ")}`,
    ).toBe(true);
  },
);

Then(
  "правило офлайн-баннера объявляет env safe-area-inset-top",
  async ({ page }) => {
    const rules = await cssRules(page, ".connectivity-banner");
    expect(rules, ".connectivity-banner rule must exist").not.toHaveLength(0);
    expect(
      rules.some((r) => r.includes("env(safe-area-inset-top")),
      `the banner rule must declare env(safe-area-inset-top), got: ${rules.join(" | ")}`,
    ).toBe(true);
  },
);

When("сеть отключается", async ({ context, page }) => {
  await context.setOffline(true);
  // Chromium may not dispatch the window event for context-level emulation
  // on every page state; the app subscribes to window offline/online —
  // dispatch explicitly so the step tests exactly that subscription.
  await page.evaluate(() => window.dispatchEvent(new Event("offline")));
});

When("сеть возвращается", async ({ context, page }) => {
  await context.setOffline(false);
  await page.evaluate(() => window.dispatchEvent(new Event("online")));
});

Then("офлайн-баннер отображается", async ({ page }) => {
  // The banner mounts without a data-testid (ConnectivityBanner is rendered
  // with an empty test_id signal), so the CSS class is the only marker.
  await expect(page.locator(".connectivity-banner")).toBeVisible({
    timeout: 15_000,
  });
});

Then("офлайн-баннер резервирует верх под статус-бар", async ({ page }) => {
  const padTop = await page
    .locator(".connectivity-banner")
    .evaluate((el) => parseFloat(getComputedStyle(el).paddingTop));
  expect(
    padTop,
    "the banner text must sit below the status bar (>= its 8px base padding)",
  ).toBeGreaterThanOrEqual(8);
});

Then("офлайн-баннер исчезает", async ({ page }) => {
  await expect(page.locator(".connectivity-banner")).toBeHidden({
    timeout: 15_000,
  });
});
