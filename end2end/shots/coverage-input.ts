import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";

import { listDescriptors } from "./catalog.ts";
import { repoRoot } from "./config.ts";
import { parseRoutePatterns } from "./coverage-checks.ts";
import { inventory } from "./coverage-inventory.ts";
import type { StaticCheckInput } from "./coverage-types.ts";

/** One source of the static-check input for the CLI and the unit suite. */
export async function buildStaticInput(): Promise<StaticCheckInput> {
  const routes = await readFile(
    join(repoRoot, "origa_ui/src/routes.rs"),
    "utf-8",
  );
  const model = JSON.parse(
    await readFile(
      join(repoRoot, "end2end/shots/examiner/model.json"),
      "utf-8",
    ),
  ) as { screens: Record<string, unknown> };
  const scenarioFiles = (
    await readdir(join(repoRoot, "end2end/shots/scenarios"))
  ).filter((name) => name.endsWith(".ts") && name !== "index.ts");
  return {
    rows: inventory,
    routePatterns: parseRoutePatterns(routes),
    scenarioIds: listDescriptors().map((descriptor) => descriptor.id),
    scenarioFiles,
    examKeys: Object.keys(model.screens),
  };
}
