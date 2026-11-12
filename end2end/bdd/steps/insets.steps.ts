import { expect, type Page } from "@playwright/test";
import { When, Then } from "../fixtures";

/**
 * Steps for the edge-to-edge insets feature (task 20261109231215-gf1g0c1).
 *
 * The device inset itself is 0 in a headless desktop browser, so the runtime
 * behaviour is pinned through the two facts that a real device combines:
 * the shell class chosen per route (`pb-safe-b` vs `app-main-with-nav`) and
 * the CSS rules actually present in the served stylesheet (the tailwind
 * utility declaring `env(safe-area-inset-bottom)` and the banner's
 * `env(safe-area-inset-top)` padding). If the utility is ever dropped from
 * the generated CSS, the rule assertion fails even though the class still
 * renders "nothing" on desktop.
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
