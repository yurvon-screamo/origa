import { resolve, sep } from "node:path";

import type { CoverageIssue, CoverageRow } from "./coverage-types.ts";
import { matchesScreen } from "./coverage-checks.ts";
import {
  readProof,
  sha256Hex,
  type ShotEntry,
  type ShotProof,
} from "./manifest.ts";

/** Everything the gate needs from the outside — no git, no browser inside. */
export interface ProofContext {
  readonly expectedCommit: string;
  readonly isAncestor: (commit: string, head: string) => boolean;
  readonly readPng: (file: string) => Promise<Buffer>;
  readonly proofRoot: string;
}

function frameMatches(
  entry: ShotEntry,
  row: CoverageRow,
  screen: string,
  viewport: string,
): boolean {
  return (
    entry.scenario === row.scenario &&
    entry.viewport === viewport &&
    matchesScreen(screen, entry.screen)
  );
}

/** Compares a real capture with the inventory; every external fact is injected. */
export async function compareManifest(
  rows: readonly CoverageRow[],
  entries: readonly ShotEntry[],
  ctx: ProofContext,
): Promise<CoverageIssue[]> {
  const issues: CoverageIssue[] = [];
  const stamps = new Map<ShotEntry, ShotProof>();
  const root = resolve(ctx.proofRoot);

  for (const entry of entries) {
    if (entry.file === "") continue;
    const absolute = resolve(entry.file);
    if (absolute !== root && !absolute.startsWith(root + sep)) {
      issues.push({
        row: "manifest",
        problem: `frame outside --run: ${entry.file}`,
      });
      continue;
    }
    let png: Buffer;
    try {
      png = await ctx.readPng(absolute);
    } catch {
      issues.push({ row: "manifest", problem: `missing frame: ${entry.file}` });
      continue;
    }
    if (sha256Hex(png) !== entry.sha256) {
      issues.push({
        row: "manifest",
        problem: `sha256 mismatch: ${entry.file}`,
      });
      continue;
    }
    let proof: ShotProof | null;
    try {
      proof = readProof(png);
    } catch {
      issues.push({
        row: "manifest",
        problem: `corrupt proof chunk: ${entry.file}`,
      });
      continue;
    }
    if (proof === null) {
      issues.push({
        row: "manifest",
        problem: `frame without proof: ${entry.file}`,
      });
      continue;
    }
    if (proof.screen !== entry.screen) {
      issues.push({
        row: "manifest",
        problem: `screen stamp mismatch: ${entry.screen} ≠ ${proof.screen}`,
      });
    }
    if (
      proof.commit !== ctx.expectedCommit ||
      !ctx.isAncestor(proof.commit, ctx.expectedCommit)
    ) {
      issues.push({
        row: "manifest",
        problem: `proof commit ${proof.commit} is not an ancestor of HEAD ${ctx.expectedCommit}`,
      });
    }
    stamps.set(entry, proof);
  }

  const commits = new Set([...stamps.values()].map((proof) => proof.commit));
  if (commits.size > 1) {
    issues.push({
      row: "manifest",
      problem: `mixed commits in one proof: ${[...commits].sort().join(", ")}`,
    });
  }

  const proved = entries.filter((entry) => stamps.has(entry));
  for (const entry of entries.filter((item) => item.file === "")) {
    for (const row of rows) {
      if (
        row.status === "covered" &&
        row.scenario === entry.scenario &&
        row.screens.some((screen) => matchesScreen(screen, entry.screen))
      ) {
        issues.push({
          row: row.id,
          problem: `negative proof for screen "${entry.screen}": marker did not settle`,
        });
      }
    }
  }

  for (const row of rows) {
    if (row.status !== "covered" || !row.scenario) continue;
    for (const viewport of row.viewports) {
      for (const screen of row.screens) {
        if (
          !proved.some((entry) => frameMatches(entry, row, screen, viewport))
        ) {
          issues.push({
            row: row.id,
            problem: `no ${viewport} frame for screen "${screen}"`,
          });
        }
      }
    }
  }
  return issues;
}

/** A proof with mixed commits is invalid as a whole, not per row. */
export function provedRows(
  rows: readonly CoverageRow[],
  issues: readonly CoverageIssue[],
): string[] {
  if (issues.some((issue) => issue.row === "manifest")) return [];
  const failing = new Set(issues.map((issue) => issue.row));
  return rows
    .filter((row) => row.status === "covered" && !failing.has(row.id))
    .map((row) => row.id);
}

export function formatCoverageReport(
  rows: readonly CoverageRow[],
  issues: readonly CoverageIssue[],
  proved: readonly string[] = [],
): string {
  const covered = rows.filter((row) => row.status === "covered");
  const gaps = rows.filter((row) => row.status === "gap");
  const excluded = rows.filter((row) => row.status === "excluded");
  const lines = [
    `Инвентарь: ${rows.length} строк; декларировано покрыто ${covered.length}, дыр ${gaps.length}, исключений ${excluded.length}; доказано манифестом ${proved.length}.`,
    "",
    "Строки:",
  ];
  for (const row of covered) {
    lines.push(
      `  covered  ${row.id} — ${row.source} [${row.viewports.join(",")}]${proved.length > 0 || issues.length > 0 ? (proved.includes(row.id) ? " · кадры есть" : " · кадров нет") : " · доказательство не проверено"}`,
    );
  }
  for (const row of gaps) {
    lines.push(
      `  gap      ${row.id} — ${row.source} [${row.viewports.join(",")}]`,
    );
  }
  for (const row of excluded) {
    lines.push(
      `  excluded ${row.id} — ${row.exclusion?.code}: ${row.exclusion?.reason}`,
    );
  }
  const questions = rows.filter(
    (row) => row.exclusion?.code === "owner-question",
  );
  if (questions.length > 0) {
    lines.push("", "Вопросы владельцу:");
    for (const row of questions) {
      lines.push(`  ${row.id}: ${row.exclusion?.reason}`);
    }
  }
  if (issues.length > 0) {
    lines.push("", `Проблемы (${issues.length}):`);
    for (const issue of issues) {
      lines.push(`  ${issue.row}: ${issue.problem}`);
    }
  } else {
    lines.push("", "Структурных проблем нет.");
  }
  return `${lines.join("\n")}\n`;
}
