import { copyFile, mkdir } from "node:fs/promises";
import { join } from "node:path";

import { repoRoot } from "../shots/config.ts";
import { buildStaticInput } from "../shots/coverage-input.ts";
import { inventory } from "../shots/coverage-inventory.ts";
import {
  checkStatic,
  compareManifest,
  formatCoverageReport,
} from "../shots/coverage.ts";
import { repoImageName, type ShotEntry } from "../shots/manifest.ts";

export interface CommandOptions {
  readonly run?: string;
  readonly out?: string;
  readonly label: string;
}

export type ManifestReader = (dir: string) => Promise<ShotEntry[]>;

export async function runCoverage(
  options: CommandOptions,
  readManifest: ManifestReader,
): Promise<number> {
  const issues = checkStatic(await buildStaticInput());
  let proved: string[] = [];
  if (options.run) {
    const manifestIssues = compareManifest(
      inventory,
      await readManifest(options.run),
    );
    issues.push(...manifestIssues);
    const failing = new Set(manifestIssues.map((issue) => issue.row));
    proved = inventory
      .filter((row) => row.status === "covered" && !failing.has(row.id))
      .map((row) => row.id);
  }
  console.log(formatCoverageReport(inventory, issues, proved));
  return issues.length > 0 ? 1 : 0;
}

export async function runStore(
  options: CommandOptions,
  readManifest: ManifestReader,
): Promise<number> {
  const from =
    options.run ?? options.out ?? join("/tmp/opencode/shots", options.label);
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
