import { WordsPage } from "../../pages/words.page.ts";
import { fresh } from "./fresh.ts";
import type { PresetContext } from "./index.ts";

/** The word of the demo profile. */
const DEMO_WORD = "私は本を読みます";

/**
 * A profile with exactly one word.
 *
 * The hand of the lesson is built from the due cards, and a full N4
 * import starts with the grammar and kanji cards — the word slide with
 * the example would be thousands of clicks away. One word makes the hand
 * a single slide.
 */
export async function withWord(context: PresetContext): Promise<void> {
    await fresh(context);

    const words = new WordsPage(context.page);
    await context.page.goto("/words");
    await words.wordsPage.waitFor({ timeout: 30_000 });
    await words.addButton.click();
    await words.drawerTextarea.fill(DEMO_WORD);
    await words.drawerAnalyzeBtn.click();
    await context.page.getByText(/Найдено/).waitFor({ timeout: 15_000 });

    const items = words.analyzedWordItems;
    await items.first().waitFor({ state: "visible", timeout: 10_000 });
    // analyze_text() pre-selects everything; deselect all, then pick the
    // first word (the disabled ones are skipped).
    const count = await items.count();
    for (let index = 0; index < count; index++) {
        const item = items.nth(index);
        const checkbox = item.locator('input[type="checkbox"]');
        if (await checkbox.isChecked().catch(() => false)) {
            const isDisabled = await item
                .evaluate((element) => element.classList.contains("cursor-not-allowed"))
                .catch(() => false);
            if (isDisabled) continue;
            await item.click();
            await context.page.waitForTimeout(150);
        }
    }
    await items.first().click();
    await context.page.waitForTimeout(200);
    await words.drawerAddBtn.click();
    await words.wordsGrid.waitFor({ timeout: 20_000 });
}
