import { descriptors } from "../catalog.ts";
import type { ShotScenario } from "../scenario.ts";

/**
 * The JFT-Basic readiness card on the home screen and its badge on the
 * sets screen.
 *
 * The profile behind the screens imported the Irodori Nyuumon lessons
 * only (see the `irodori-nyuumon` preset) — that is what the readiness
 * card reads.
 */
export const jftCardScenario: ShotScenario = {
    ...descriptors["jft-card"],
    async run({ page }, sink) {
        // The readiness card needs the CDN catalog fetch; give it a
        // generous window.
        await page.getByTestId("home-jft-readiness").waitFor({ state: "visible", timeout: 60_000 });
        await page
            .getByTestId("home-jft-readiness-stamp")
            .waitFor({ state: "visible", timeout: 10_000 });
        await sink.shot("home-jft");

        // Sets page: filter to Irodori so the badge cards fill the viewport.
        await page.goto("/sets", { waitUntil: "domcontentloaded" });
        await page.getByTestId("sets-search-input").waitFor({ state: "visible", timeout: 60_000 });
        await page.getByTestId("sets-search-input").fill("Irodori");
        await page.getByTestId("sets-loading").waitFor({ state: "hidden", timeout: 60_000 });
        await page
            .getByTestId("sets-card-jft-badge")
            .first()
            .waitFor({ state: "visible", timeout: 60_000 });
        await sink.shot("sets-jft-badge");
    },
};
