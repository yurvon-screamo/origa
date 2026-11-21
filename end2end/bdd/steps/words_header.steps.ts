import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { WordsPage } from "../../pages";

/**
 * The words header layout contract (bug: «заголовок/табы/кнопка разъехались
 * в 2 ряда»): below the `md` breakpoint (768px) the actions wrapper takes
 * its own full-width row (`max-md:basis-full max-md:justify-between` — a
 * deliberate tab strip under the title), at or above it the header stays
 * one line. The classes come from words/header.rs through the PageHeader
 * `actions_class` prop; the wasm unit anchor
 * (`words_header_renders_title_and_actions`) pins the class string, these
 * scenarios pin the computed layout a user actually sees.
 */

const MOBILE = { width: 390, height: 844 };
const DESKTOP = { width: 1280, height: 720 };

Given("шапка экрана слов открыта", async ({ page }) => {
  const wordsPage = new WordsPage(page);
  await wordsPage.goto();
  await wordsPage.expectWordsVisible();
  await expect(wordsPage.wordsActions).toBeVisible();
});

When("пользователь сужает окно до мобильной ширины", async ({ page }) => {
  await page.setViewportSize(MOBILE);
});

When("пользователь возвращает окно до десктопной ширины", async ({ page }) => {
  await page.setViewportSize(DESKTOP);
});

Then(
  "полоса действий шапки слов занимает собственную строку",
  async ({ page }) => {
    const wordsPage = new WordsPage(page);
    const actions = wordsPage.wordsActions;
    await expect(actions).toBeVisible();

    // The deliberate-grid classes resolve to a full-width flex-basis below
    // the md breakpoint; a regression that drops `max-md:basis-full` leaves
    // `auto` and the accidental right-hugging wrap returns.
    await expect(actions).toHaveCSS("flex-basis", "100%", { timeout: 10_000 });

    const titleBox = await wordsPage.wordsTitle.boundingBox();
    const actionsBox = await actions.boundingBox();
    expect(titleBox, "words-title должен быть отрендерен").not.toBeNull();
    expect(actionsBox, "words-actions должен быть отрендерен").not.toBeNull();
    // Row 2: the tab strip starts strictly below the title row.
    expect(actionsBox!.y).toBeGreaterThanOrEqual(
      titleBox!.y + titleBox!.height,
    );
    // Full-width strip: it stretches across the card content, not hugging right.
    expect(actionsBox!.width).toBeGreaterThan(titleBox!.width);
  },
);

Then(
  "полоса действий шапки слов стоит в одной строке с заголовком",
  async ({ page }) => {
    const wordsPage = new WordsPage(page);
    const actions = wordsPage.wordsActions;
    await expect(actions).toBeVisible();

    const titleBox = await wordsPage.wordsTitle.boundingBox();
    const actionsBox = await actions.boundingBox();
    expect(titleBox).not.toBeNull();
    expect(actionsBox).not.toBeNull();
    // Same row: tops nearly level (the flex container centers items).
    expect(Math.abs(actionsBox!.y - titleBox!.y)).toBeLessThan(8);
    // NOT the mobile full-basis: the deliberate grid must relax above md.
    await expect(actions).not.toHaveCSS("flex-basis", "100%", {
      timeout: 10_000,
    });
  },
);
