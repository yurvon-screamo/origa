import type { ShotScenario } from "../scenario.ts";
import { homeScenario } from "./home.ts";

/**
 * The browser-side implementations of the catalog.
 *
 * The metadata lives in `shots/catalog.ts` — this map only adds the
 * steps that reach the screen.
 */
export const implementations: Record<string, ShotScenario> = {
    [homeScenario.id]: homeScenario,
};

export function getImplementation(id: string): ShotScenario {
    const scenario = implementations[id];
    if (!scenario) {
        throw new Error(
            `Unknown scenario "${id}". Known: ${Object.keys(implementations).sort().join(", ")}`,
        );
    }
    return scenario;
}
