import { execFileSync } from "node:child_process";
import { copyFile, mkdir, readFile } from "node:fs/promises";
import { join } from "node:path";

import { repoRoot } from "../shots/config.ts";
import { buildStaticInput } from "../shots/coverage-input.ts";
import { inventory } from "../shots/coverage-inventory.ts";
import type { ProofContext } from "../shots/coverage-report.ts";
import {
  checkStatic,
  compareManifest,
  formatCoverageReport,
  provedRows,
} from "../shots/coverage.ts";
import { repoImageName, type ShotEntry } from "../shots/manifest.ts";

export interface CommandOptions {
  readonly run?: string;
  readonly out?: string;
}

export type ManifestReader = (dir: string) => Promise<ShotEntry[]>;

/** The gate's external facts: checkout HEAD and git ancestry, never the manifest. */
export function localProofContext(runDir: string): ProofContext {
  const head = (): string =>
    execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: repoRoot,
      encoding: "utf-8",
    }).trim();
  const expectedCommit = head();
  return {
    expectedCommit,
    isAncestor: (commit: string, expected: string): boolean => {
      try {
        execFileSync("git", ["merge-base", "--is-ancestor", commit, expected], {
          cwd: repoRoot,
          stdio: "ignore",
        });
        return true;
      } catch {
        return false;
      }
    },
    readPng: (file: string) => readFile(file),
    proofRoot: runDir,
  };
}

export async function runCoverage(
  options: CommandOptions,
  readManifest: ManifestReader,
  proofContext?: ProofContext,
): Promise<number> {
  const issues = checkStatic(await buildStaticInput());
  let proved: string[] = [];
  if (options.run) {
    if (!proofContext) throw new Error("--run needs an external proof context");
    const manifestIssues = await compareManifest(
      inventory,
      await readManifest(options.run),
      proofContext,
    );
    issues.push(...manifestIssues);
    proved = provedRows(inventory, manifestIssues);
  }
  console.log(formatCoverageReport(inventory, issues, proved));
  return issues.length > 0 ? 1 : 0;
}

export async function runStore(
  options: CommandOptions,
  readManifest: ManifestReader,
): Promise<number> {
  const from = options.run ?? options.out;
  if (!from) throw new Error("store needs --out or --run");
  const entries = (await readManifest(from)).filter(
    (entry) => entry.file !== "",
  );
  const target = join(repoRoot, "docs/ui-screenshots/images");
  await mkdir(target, { recursive: true });
  for (const entry of entries) {
    await copyFile(entry.file, join(target, repoImageName(entry)));
  }
  console.log(`В репозиторий скопировано кадров: ${entries.length}`);
  console.log(`Каталог: ${target}`);
  return 0;
}
