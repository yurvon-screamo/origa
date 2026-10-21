import type { ScenarioDescriptor } from "./catalog.ts";
import type { ShotContext, ShotSink } from "./types.ts";

/**
 * A screen of the app, reproducible.
 *
 * A scenario describes HOW to reach a screen and WHAT to keep; the tool
 * owns the browser session, the files and the bookkeeping.
 */
export type ShotScenario = ScenarioDescriptor & {
    /** Reaches the screen and puts the wanted captures into the sink. */
    run(context: ShotContext, sink: ShotSink): Promise<void>;
};
