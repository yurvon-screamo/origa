import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The home screen with an empty corpus.
 *
 * The first screen a user sees — the one every pull request that touches
 * the layout has to show.
 */
export const homeScenario: ShotScenario = {
    ...descriptors.home,
    async run({ page }, sink) {
        await page.goto("/home");
        await page.getByTestId("home-page").waitFor({ state: "visible", timeout: 60_000 });
        await sink.shot("home");
    },
};
