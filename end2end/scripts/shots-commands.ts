import { copyFile, mkdir, readFile, readdir } from "node:fs/promises";
import { join } from "node:path";

import { listDescriptors } from "../shots/catalog.ts";
import { repoRoot } from "../shots/config.ts";
import { inventory } from "../shots/coverage-inventory.ts";
import {
  checkStatic,
  compareManifest,
  formatCoverageReport,
  parseRoutePatterns,
} from "../shots/coverage.ts";
import { repoImageName, type ShotEntry } from "../shots/manifest.ts";

export interface CommandOptions {
  readonly run?: string;
  readonly label: string;
}

export type ManifestReader = (dir: string) => Promise<ShotEntry[]>;

export async function runCoverage(
  options: CommandOptions,
  readManifest: ManifestReader,
): Promise<number> {
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
  const issues = checkStatic({
    rows: inventory,
    routePatterns: parseRoutePatterns(routes),
    scenarioIds: listDescriptors().map((descriptor) => descriptor.id),
    scenarioFiles,
    examKeys: Object.keys(model.screens),
  });
  if (options.run) {
    issues.push(...compareManifest(inventory, await readManifest(options.run)));
  }
  console.log(formatCoverageReport(inventory, issues));
  return issues.length > 0 ? 1 : 0;
}

export async function runStore(
  options: CommandOptions,
  readManifest: ManifestReader,
): Promise<number> {
  const from = options.run ?? join("/tmp/opencode/shots", options.label);
  const entries = await readManifest(from);
  const target = join(repoRoot, "docs/ui-screenshots/images");
  await mkdir(target, { recursive: true });
  for (const entry of entries) {
    await copyFile(entry.file, join(target, repoImageName(entry)));
  }
  console.log(`В репозиторий скопировано кадров: ${entries.length}`);
  console.log(`Каталог: ${target}`);
  return 0;
}
